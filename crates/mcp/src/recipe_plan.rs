//! Deterministic, game-free arithmetic over explicitly supplied recipes.

use std::collections::{BTreeMap, BTreeSet};

use policy::mcp::Tool;
use policy::recipe_plan::{self as admission, Request};
use serde_json::{Map, Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "recipe_plan",
        description: "Report-only crafting arithmetic over caller-supplied recipes, stock, goals, available stations and unlocks. Multiple producers require explicit item/recipe choices; selected multi-output recipes share batches across their assigned outputs, with unassigned coproducts retained as surplus. No automatic optimization. Returns batch-rounded steps, stock reservations, shortages and unmet requirements. Does not read the game, verify the catalogue, craft, gather or authorize writes.",
        schema: admission::schema(),
    }
}

pub(crate) fn call(args: &Map<String, Value>) -> Result<Value, String> {
    plan(admission::check(args)?)
}

fn plan(request: Request) -> Result<Value, String> {
    let mut demand = request.goals.clone();
    let mut stock_used = BTreeMap::new();
    let mut remaining_stock = request.stock.clone();
    let mut surplus = BTreeMap::new();
    let mut shortages = BTreeMap::new();
    let mut missing_stations = BTreeSet::new();
    let mut missing_unlocks = BTreeSet::new();
    let mut steps = Vec::new();
    let bundles = request.recipes.values().any(|recipe| recipe.bundle);
    for group in &request.order {
        let mut needs = BTreeMap::new();
        for item in group {
            let required = demand.get(item).copied().unwrap_or(0);
            let available = request.stock.get(item).copied().unwrap_or(0);
            let used = available.min(required);
            if used > 0 {
                stock_used.insert(item.clone(), used);
                remaining_stock.insert(item.clone(), available - used);
            }
            if required > used {
                needs.insert(item.clone(), required - used);
            }
        }
        if needs.is_empty() {
            continue;
        }
        let Some(recipe) = request.recipes.get(&group[0]) else {
            shortages.extend(needs);
            continue;
        };
        let batches = needs
            .iter()
            .map(|(item, needed)| needed.div_ceil(recipe.outputs[item]))
            .max()
            .expect("the group needs at least one output");
        let mut outputs = BTreeMap::new();
        for (item, per_batch) in &recipe.outputs {
            let produced = admission::multiply(batches, *per_batch)?;
            outputs.insert(item.clone(), produced);
            let needed = needs.get(item).copied().unwrap_or(0);
            if produced > needed {
                let old = surplus.get(item).copied().unwrap_or(0);
                surplus.insert(item.clone(), admission::add(old, produced - needed)?);
            }
        }
        let mut ingredients = BTreeMap::new();
        for (ingredient, per_batch) in &recipe.ingredients {
            let quantity = admission::multiply(batches, *per_batch)?;
            let old = demand.get(ingredient).copied().unwrap_or(0);
            demand.insert(ingredient.clone(), admission::add(old, quantity)?);
            ingredients.insert(ingredient.clone(), quantity);
        }
        let stations: Vec<_> = recipe
            .stations
            .difference(&request.stations)
            .cloned()
            .collect();
        let unlocks: Vec<_> = recipe
            .unlocks
            .difference(&request.unlocks)
            .cloned()
            .collect();
        missing_stations.extend(stations.iter().cloned());
        missing_unlocks.extend(unlocks.iter().cloned());
        let mut step = json!({
            "recipe":recipe.id,"batches":batches,"ingredients":quantities(&ingredients),
            "required_stations":recipe.stations,"required_unlocks":recipe.unlocks,
            "missing_stations":stations,"missing_unlocks":unlocks
        });
        if recipe.bundle {
            step["outputs"] = json!(quantities(&outputs));
            step["needed_after_stock"] = json!(quantities(&needs));
        } else {
            let item = &group[0];
            step["item"] = json!(item);
            step["produced"] = json!(outputs[item]);
            step["needed_after_stock"] = json!(needs[item]);
        }
        steps.push(step);
    }
    // Dependencies were expanded from outputs toward ingredients. Craft in
    // reverse order so every crafted ingredient precedes its consumers.
    steps.reverse();
    remaining_stock.retain(|_, count| *count != 0);
    let feasible =
        shortages.is_empty() && missing_stations.is_empty() && missing_unlocks.is_empty();
    let mut result = json!({
        "mode":"report_only",
        "catalog_label":request.catalog_label,
        "provenance":"caller_supplied_unverified",
        "feasible_under_supplied_assumptions":feasible,
        "goals":quantities(&request.goals),
        "steps":steps,
        "stock_reserved":quantities(&stock_used),
        "unreserved_stock":quantities(&remaining_stock),
        "planned_surplus":quantities(&surplus),
        "missing_raw_items":quantities(&shortages),
        "missing_stations":missing_stations,
        "missing_unlocks":missing_unlocks,
        "limits":[
            "Inputs are supplied by the caller; no live stock, recipe, station or unlock state is verified.",
            "One selected single-output recipe per item; alternatives require explicit choices, with no automatic search or optimality claim. Ingredient categories, coproducts, recycling cycles and catalysts are unsupported.",
            "Requirements are names supplied by the caller, not inferred game rules. Missing stations and unlocks are prerequisites, not automatically crafted or granted.",
            "No costs, durations, inventory eligibility, reservations in the game, progression side effects or optimality are inferred."
        ]
    });
    if bundles {
        result["output_allocation"] = json!("selected_producers_only");
        result["selected_producers"] = json!(
            request
                .recipes
                .iter()
                .map(|(item, recipe)| json!({"item":item,"recipe":recipe.id}))
                .collect::<Vec<_>>()
        );
        result["limits"][1] = json!(
            "One selected producer per item; outputs assigned to the same recipe share batches. Coproducts assigned to another producer remain surplus and never silently replace that choice. No automatic search or optimality claim. Ingredient categories, recycling cycles and catalysts are unsupported."
        );
    }
    Ok(result)
}

fn quantities(items: &BTreeMap<String, u64>) -> Vec<Value> {
    items
        .iter()
        .map(|(item, count)| json!({"item":item,"count":count}))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recipe(id: &str, item: &str, count: u64, ingredients: Value) -> Value {
        json!({"id":id,"item":item,"count":count,"ingredients":ingredients,"stations":[],"unlocks":[]})
    }

    fn run(recipes: Value, stock: Value, goals: Value) -> Result<Value, String> {
        call(
            json!({"catalog_label":"synthetic test data","recipes":recipes,
            "stock":stock,"goals":goals,"stations":[],"unlocks":[]})
            .as_object()
            .unwrap(),
        )
    }

    fn bundle() -> Value {
        json!({"id":"saw","outputs":[{"item":"board","count":3},{"item":"scrap","count":2}],
            "ingredients":[{"item":"wood","count":4}],"stations":[],"unlocks":[]})
    }

    #[test]
    fn output_bundle_shares_batches_and_stock_across_goals() {
        let result = run(json!([bundle()]),
            json!([{"item":"board","count":1},{"item":"scrap","count":1},{"item":"wood","count":8}]),
            json!([{"item":"board","count":5},{"item":"scrap","count":4}])).unwrap();
        assert_eq!(result["feasible_under_supplied_assumptions"], true);
        assert_eq!(result["steps"].as_array().unwrap().len(), 1);
        assert_eq!(result["steps"][0]["batches"], 2);
        assert_eq!(
            result["steps"][0]["outputs"],
            json!([{"item":"board","count":6},{"item":"scrap","count":4}])
        );
        assert_eq!(
            result["planned_surplus"],
            json!([{"item":"board","count":2},{"item":"scrap","count":1}])
        );
        assert_eq!(result["unreserved_stock"], json!([]));
        assert_eq!(
            result["stock_reserved"],
            json!([{"item":"board","count":1},{"item":"scrap","count":1},{"item":"wood","count":8}])
        );
    }

    #[test]
    fn bundle_dependencies_wait_for_all_consumers_and_preserve_input_order_invariance() {
        let chest = recipe(
            "chest",
            "chest",
            1,
            json!([{"item":"board","count":4},{"item":"scrap","count":3}]),
        );
        let goals = json!([{"item":"chest","count":1},{"item":"board","count":1}]);
        let first = run(json!([bundle(), chest]), json!([]), goals.clone()).unwrap();
        assert_eq!(first["steps"][0]["recipe"], "saw");
        assert_eq!(first["steps"][0]["batches"], 2);
        assert_eq!(first["steps"][1]["recipe"], "chest");
        assert_eq!(
            first["missing_raw_items"],
            json!([{"item":"wood","count":8}])
        );
        let mut reversed = bundle();
        reversed["outputs"].as_array_mut().unwrap().reverse();
        assert_eq!(
            run(json!([chest, reversed]), json!([]), goals).unwrap(),
            first
        );
    }

    #[test]
    fn shared_bundle_waits_for_consumers_at_different_dependency_depths() {
        let recipes = json!([
            bundle(),
            recipe("a", "a", 1, json!([{"item":"board","count":2}])),
            recipe("z", "z", 1, json!([{"item":"middle","count":1}])),
            recipe("middle", "middle", 1, json!([{"item":"scrap","count":5}]))
        ]);
        let result = run(
            recipes,
            json!([]),
            json!([{"item":"a","count":1},{"item":"z","count":1}]),
        )
        .unwrap();
        assert_eq!(result["steps"][0]["recipe"], "saw");
        assert_eq!(result["steps"][0]["batches"], 3);
        assert_eq!(
            result["missing_raw_items"],
            json!([{"item":"wood","count":12}])
        );
        assert_eq!(
            result["planned_surplus"],
            json!([{"item":"board","count":7},{"item":"scrap","count":1}])
        );
    }

    #[test]
    fn installed_legacy_synthetic_contract_is_unchanged() {
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/recipe-plan-legacy.json"))
                .unwrap();
        assert_eq!(
            call(fixture["request"].as_object().unwrap()).unwrap(),
            fixture["response"]
        );
    }

    #[test]
    fn overlapping_outputs_preserve_explicit_producers_and_add_surplus() {
        let mut args = json!({"catalog_label":"synthetic overlapping producers",
            "recipes":[bundle(),recipe("scrap-only","scrap",2,json!([{"item":"ore","count":1}]))],
            "stock":[],"goals":[{"item":"board","count":1},{"item":"scrap","count":1}],
            "stations":[],"unlocks":[],"choices":[{"item":"scrap","recipe":"scrap-only"}]});
        let result = call(args.as_object().unwrap()).unwrap();
        assert_eq!(result["steps"].as_array().unwrap().len(), 2);
        assert_eq!(result["output_allocation"], "selected_producers_only");
        assert_eq!(
            result["planned_surplus"],
            json!([{"item":"board","count":2},{"item":"scrap","count":3}])
        );
        assert_eq!(
            result["missing_raw_items"],
            json!([{"item":"ore","count":1},{"item":"wood","count":4}])
        );
        args["choices"][0]["recipe"] = json!("saw");
        let result = call(args.as_object().unwrap()).unwrap();
        assert_eq!(result["steps"].as_array().unwrap().len(), 1);
        assert_eq!(
            result["missing_raw_items"],
            json!([{"item":"wood","count":4}])
        );
        let mut oversized = bundle();
        oversized["outputs"][0]["count"] = json!(admission::MAX_QUANTITY);
        assert!(
            run(
                json!([oversized]),
                json!([]),
                json!([{"item":"scrap","count":1_000_000_000}])
            )
            .unwrap_err()
            .contains("limit")
        );
    }

    #[test]
    fn independent_enumeration_checks_bundle_batches_and_inventory_conservation() {
        let map = |rows: &Value| -> BTreeMap<String, u64> {
            rows.as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        r["item"].as_str().unwrap().into(),
                        r["count"].as_u64().unwrap(),
                    )
                })
                .collect()
        };
        for a in 1..=3u64 {
            for b in 1..=3u64 {
                for goal_a in 1..=4u64 {
                    for goal_b in 1..=4u64 {
                        for stock_a in 0..=2u64 {
                            for stock_b in 0..=2u64 {
                                // Search batch counts instead of repeating the planner's rounding formula.
                                let batches = (0..=8u64)
                                    .find(|n| {
                                        stock_a + n * a >= goal_a && stock_b + n * b >= goal_b
                                    })
                                    .unwrap();
                                let mut r = bundle();
                                r["outputs"] =
                                    json!([{"item":"a","count":a},{"item":"b","count":b}]);
                                r["ingredients"] = json!([{"item":"wood","count":2}]);
                                let mut inventory = BTreeMap::from([
                                    ("wood".to_owned(), 100u64),
                                    ("a".into(), stock_a),
                                    ("b".into(), stock_b),
                                ]);
                                let stock: Vec<_> = inventory
                                    .iter()
                                    .filter(|(_, n)| **n > 0)
                                    .map(|(item, count)| json!({"item":item,"count":count}))
                                    .collect();
                                let result = run(json!([r]),json!(stock),json!([{"item":"a","count":goal_a},{"item":"b","count":goal_b}])).unwrap();
                                let steps = result["steps"].as_array().unwrap();
                                assert_eq!(steps.len(), usize::from(batches > 0));
                                for step in steps {
                                    assert_eq!(step["batches"], batches);
                                    for (item, count) in map(&step["ingredients"]) {
                                        *inventory.get_mut(&item).unwrap() =
                                            inventory[&item].checked_sub(count).unwrap();
                                    }
                                    for (item, count) in map(&step["outputs"]) {
                                        *inventory.entry(item).or_default() += count;
                                    }
                                }
                                *inventory.get_mut("a").unwrap() -= goal_a;
                                *inventory.get_mut("b").unwrap() -= goal_b;
                                inventory.retain(|_, n| *n > 0);
                                let mut reported = map(&result["unreserved_stock"]);
                                for (item, count) in map(&result["planned_surplus"]) {
                                    *reported.entry(item).or_default() += count;
                                }
                                assert_eq!(inventory, reported);
                                assert_eq!(
                                    map(&result["stock_reserved"])
                                        .get("wood")
                                        .copied()
                                        .unwrap_or(0),
                                    2 * batches
                                );
                                assert_eq!(result["feasible_under_supplied_assumptions"], true);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn explicit_alternative_controls_batches_inputs_and_requirements() {
        let mut slow = recipe("slow", "board", 1, json!([{"item":"wood","count":3}]));
        slow["stations"] = json!(["unavailable station"]);
        let fast = recipe("fast", "board", 3, json!([{"item":"scrap","count":2}]));
        let mut args = json!({"catalog_label":"synthetic alternatives","recipes":[slow,fast],
            "stock":[{"item":"scrap","count":4}],"goals":[{"item":"board","count":4}],
            "stations":[],"unlocks":[],"choices":[{"item":"board","recipe":"fast"}]});
        let result = call(args.as_object().unwrap()).unwrap();
        assert_eq!(result["feasible_under_supplied_assumptions"], true);
        assert_eq!(result["steps"][0]["recipe"], "fast");
        assert_eq!(result["steps"][0]["batches"], 2);
        assert_eq!(
            result["stock_reserved"],
            json!([{"item":"scrap","count":4}])
        );
        assert_eq!(
            result["planned_surplus"],
            json!([{"item":"board","count":2}])
        );
        args["recipes"].as_array_mut().unwrap().reverse();
        assert_eq!(call(args.as_object().unwrap()).unwrap(), result);
        args["choices"][0]["recipe"] = json!("slow");
        let result = call(args.as_object().unwrap()).unwrap();
        assert_eq!(
            result["missing_raw_items"],
            json!([{"item":"wood","count":12}])
        );
        assert_eq!(result["missing_stations"], json!(["unavailable station"]));
    }

    #[test]
    fn shared_ingredients_are_aggregated_before_batch_rounding() {
        let recipes = json!([
            recipe("make_a", "a", 1, json!([{"item":"board","count":1}])),
            recipe("make_b", "b", 1, json!([{"item":"board","count":1}])),
            recipe("saw", "board", 3, json!([{"item":"wood","count":2}]))
        ]);
        let result = run(
            recipes,
            json!([{"item":"wood","count":2}]),
            json!([{"item":"a","count":1},{"item":"b","count":1}]),
        )
        .unwrap();
        assert_eq!(result["feasible_under_supplied_assumptions"], true);
        assert_eq!(result["steps"][0]["recipe"], "saw");
        assert_eq!(result["steps"][0]["batches"], 1);
        assert_eq!(
            result["planned_surplus"],
            json!([{"item":"board","count":1}])
        );
        assert_eq!(result["stock_reserved"], json!([{"item":"wood","count":2}]));
    }

    #[test]
    fn finished_stock_short_circuits_production_and_requirements() {
        let mut r = recipe("assemble", "chest", 1, json!([{"item":"wood","count":20}]));
        r["stations"] = json!(["workbench"]);
        let result = run(
            json!([r]),
            json!([{"item":"chest","count":2}]),
            json!([{"item":"chest","count":1}]),
        )
        .unwrap();
        assert_eq!(result["steps"], json!([]));
        assert_eq!(result["missing_stations"], json!([]));
        assert_eq!(
            result["unreserved_stock"],
            json!([{"item":"chest","count":1}])
        );
    }

    #[test]
    fn goals_and_ingredients_share_inventory_once() {
        let recipes = json!([recipe(
            "assemble",
            "chest",
            1,
            json!([{"item":"wood","count":3}])
        )]);
        let result = run(
            recipes,
            json!([{"item":"wood","count":4}]),
            json!([{"item":"chest","count":1},{"item":"wood","count":2}]),
        )
        .unwrap();
        assert_eq!(
            result["missing_raw_items"],
            json!([{"item":"wood","count":1}])
        );
        assert_eq!(result["stock_reserved"], json!([{"item":"wood","count":4}]));
        assert_eq!(result["feasible_under_supplied_assumptions"], false);
    }

    #[test]
    fn missing_requirements_and_inputs_remain_explicit() {
        let mut r = recipe("assemble", "chest", 1, json!([{"item":"wood","count":3}]));
        r["stations"] = json!(["workbench"]);
        r["unlocks"] = json!(["chest_recipe"]);
        let result = run(json!([r]), json!([]), json!([{"item":"chest","count":1}])).unwrap();
        assert_eq!(result["missing_stations"], json!(["workbench"]));
        assert_eq!(result["missing_unlocks"], json!(["chest_recipe"]));
        assert_eq!(
            result["missing_raw_items"],
            json!([{"item":"wood","count":3}])
        );
        assert_eq!(result["provenance"], "caller_supplied_unverified");
    }

    #[test]
    fn cycles_and_excessive_expansion_return_errors() {
        let cyclic = json!([recipe("loop", "a", 1, json!([{"item":"a","count":1}]))]);
        assert!(
            run(cyclic, json!([]), json!([{"item":"a","count":1}]))
                .unwrap_err()
                .contains("cycle")
        );
        let huge = json!([recipe(
            "huge",
            "a",
            1,
            json!([{"item":"b","count":admission::MAX_QUANTITY}])
        )]);
        assert!(
            run(
                huge,
                json!([]),
                json!([{"item":"a","count":admission::MAX_QUANTITY}])
            )
            .unwrap_err()
            .contains("limit")
        );
    }

    #[test]
    fn output_order_is_deterministic_and_ingredients_precede_consumers() {
        let a = recipe("a", "a", 1, json!([{"item":"b","count":1}]));
        let b = recipe("b", "b", 1, json!([{"item":"raw","count":1}]));
        let first = run(json!([a, b]), json!([]), json!([{"item":"a","count":1}])).unwrap();
        let second = run(json!([b, a]), json!([]), json!([{"item":"a","count":1}])).unwrap();
        assert_eq!(first, second);
        assert_eq!(first["steps"][0]["item"], "b");
        assert_eq!(first["steps"][1]["item"], "a");
    }
}
