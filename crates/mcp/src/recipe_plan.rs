//! Deterministic, game-free arithmetic over explicitly supplied recipes.

use std::collections::{BTreeMap, BTreeSet};

use policy::mcp::Tool;
use policy::recipe_plan::{self as admission, Request};
use serde_json::{Map, Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "recipe_plan",
        description: "Report-only crafting arithmetic over caller-supplied recipes, stock, goals, available stations and unlocks. Multiple single-output producers require explicit item/recipe choices; no automatic optimization. Returns batch-rounded steps, stock reservations, shortages and unmet requirements. Does not read the game, verify the catalogue, craft, gather or authorize writes.",
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
    for item in &request.order {
        let required = demand.get(item).copied().unwrap_or(0);
        if required == 0 {
            continue;
        }
        let available = request.stock.get(item).copied().unwrap_or(0);
        let used = available.min(required);
        if used > 0 {
            stock_used.insert(item.clone(), used);
            remaining_stock.insert(item.clone(), available - used);
        }
        let needed = required - used;
        if needed == 0 {
            continue;
        }
        let Some(recipe) = request.recipes.get(item) else {
            shortages.insert(item.clone(), needed);
            continue;
        };
        let batches = needed.div_ceil(recipe.count);
        let produced = admission::multiply(batches, recipe.count)?;
        if produced > needed {
            surplus.insert(item.clone(), produced - needed);
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
        steps.push(json!({
            "recipe":recipe.id,"item":item,"batches":batches,"produced":produced,
            "needed_after_stock":needed,"ingredients":quantities(&ingredients),
            "required_stations":recipe.stations,"required_unlocks":recipe.unlocks,
            "missing_stations":stations,"missing_unlocks":unlocks
        }));
    }
    // Dependencies were expanded from outputs toward ingredients. Craft in
    // reverse order so every crafted ingredient precedes its consumers.
    steps.reverse();
    remaining_stock.retain(|_, count| *count != 0);
    let feasible =
        shortages.is_empty() && missing_stations.is_empty() && missing_unlocks.is_empty();
    Ok(json!({
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
    }))
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
