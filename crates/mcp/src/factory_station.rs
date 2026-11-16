//! Published server factory state; raw observations never authorize crafting.
use enshwick::typed::ecs::FactoryStation;
use serde_json::{Value, json};

pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "factory_station_read",
        description: "Read one session-local server entity's published typed FactoryStation component on the game thread. Preserve running/next recipe IDs, up to four raw category choices each, raw signed times, player-stop/use-all flags and SDK missing-requirement names. Not every crafting station has this component. Unavailable differs from not observed; no complete queue, current client selection, eligibility, safe-removal, crafting execution or time-unit claim.",
        schema: policy::factory_station::schema(),
    }
}

fn observe(result: Result<Option<FactoryStation>, String>) -> Result<Value, String> {
    let mut result = match result {
        Err(error) => json!({"state":"unknown","factory":null,"error":error}),
        Ok(None) => json!({"state":"not_observed","factory":null}),
        Ok(Some(value)) => {
            let running = &value.running_recipe_input_category_selections.selections;
            let next = &value.next_recipe_input_category_selections.selections;
            policy::factory_station::counts(
                running.len(),
                next.len(),
                value.missing_requirements.0.len(),
            )?;
            let choices = |values: &[enshwick::typed::RecipeInputCategorySelection]| {
                values
                    .iter()
                    .map(|v| json!({"category_id":v.id.value,"selected_index":v.selected_index}))
                    .collect::<Vec<_>>()
            };
            json!({"state":"observed","factory":{
                "station_state":format!("{:?}",value.state),
                "running_recipe_id":value.running_recipe.value,"next_recipe_id":value.next_recipe.value,
                "running_category_choices":choices(running),"next_category_choices":choices(next),
                "recipe_start_raw":value.recipe_start.value,"recipe_pause_duration_raw":value.recipe_pause_duration.value,
                "use_all_available_category_ingredients":value.use_all_available_category_ingredients,
                "is_stopped_by_player":value.is_stopped_by_player,
                "sdk_missing_requirements":value.missing_requirements.0.iter().map(|v|format!("{v:?}")).collect::<Vec<_>>()}})
        }
    };
    result["crafting_eligibility_verified"] = json!(false);
    result["safe_removal_verified"] = json!(false);
    result["complete_queue_verified"] = json!(false);
    result["current_client_selection_verified"] = json!(false);
    Ok(result)
}

pub(crate) fn read(game: crate::tools::Game, entity: u32) -> Result<Value, String> {
    let result = game
        .factory_station
        .ok_or_else(|| "published typed FactoryStation reader unavailable".to_owned())
        .and_then(|reader| {
            reader
                .read(game.world, enshwick::Entity(entity))
                .map_err(|e| e.to_string())
        });
    let mut report = observe(result)?;
    report["source"] = json!("published_typed_server_factory_station");
    report["entity"] = json!(entity);
    report["world_key"] = json!(game.world.key());
    report["world_mutated"] = json!(false);
    report["identity_scope"] = json!(
        "server entity in current session only; reacquire after reload; never use client entity IDs"
    );
    report["limitations"] = json!([
        "not observed does not distinguish missing entity, absent component or uninitialized state",
        "raw times have no verified UI units or ETA conversion",
        "running and next fields are not a complete production queue",
        "raw category choices do not establish item membership, quantities, scaling or eligibility",
        "no observed flag authorizes crafting, state clearing, removal or undo"
    ]);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{
        resources::Value as Raw,
        typed::{Reflected, ecs::FactoryStation},
    };
    use serde_json::json;

    fn fields(values: Vec<(&str, Raw)>) -> Raw {
        Raw::Struct(values.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
    fn fixture() -> FactoryStation {
        let id = |n| fields(vec![("value", Raw::Uint(n))]);
        let choices = fields(vec![(
            "selections",
            Raw::List(vec![
                fields(vec![("id", id(0)), ("selectedIndex", Raw::Uint(255))]),
                fields(vec![
                    ("id", id(u32::MAX.into())),
                    ("selectedIndex", Raw::Uint(0)),
                ]),
            ]),
        )]);
        FactoryStation::from_value(&fields(vec![
            ("state", Raw::Name("Running".into())),
            ("runningRecipe", id(u32::MAX.into())),
            ("runningRecipeInputCategorySelections", choices.clone()),
            ("nextRecipe", id(0)),
            ("nextRecipeInputCategorySelections", choices),
            ("recipeStart", fields(vec![("value", Raw::Int(-17))])),
            ("recipePauseDuration", fields(vec![("value", Raw::Int(91))])),
            ("useAllAvailableCategoryIngredients", Raw::Bool(true)),
            ("isStoppedByPlayer", Raw::Bool(false)),
            (
                "missingRequirements",
                Raw::Flags(vec!["OutputSpace".into(), "InputMaterials".into()]),
            ),
        ]))
        .unwrap()
    }
    #[test]
    fn unavailable_absent_and_zero_recipe_fields_remain_distinct() {
        assert_eq!(
            observe(Err("reader unavailable".into())).unwrap()["state"],
            "unknown"
        );
        assert_eq!(observe(Ok(None)).unwrap()["state"], "not_observed");
        let value = observe(Ok(Some(fixture()))).unwrap();
        assert_eq!(value["state"], "observed");
        let data = &value["factory"];
        assert_eq!(data["station_state"], "Running");
        assert_eq!(data["running_recipe_id"], u32::MAX);
        assert_eq!(data["next_recipe_id"], 0);
        assert_eq!(
            data["running_category_choices"],
            json!([{"category_id":0,"selected_index":255},{"category_id":u32::MAX,"selected_index":0}])
        );
        assert_eq!(data["recipe_start_raw"], -17);
        assert_eq!(data["recipe_pause_duration_raw"], 91);
        assert_eq!(
            data["sdk_missing_requirements"],
            json!(["OutputSpace", "InputMaterials"])
        );
        assert_eq!(data["use_all_available_category_ingredients"], true);
        assert_eq!(data["is_stopped_by_player"], false);
        assert_eq!(value["crafting_eligibility_verified"], false);
        assert_eq!(value["safe_removal_verified"], false);
        assert_eq!(value["complete_queue_verified"], false);
        let mut too_many = fixture();
        too_many.next_recipe_input_category_selections.selections =
            vec![too_many.next_recipe_input_category_selections.selections[0].clone(); 5];
        assert!(observe(Ok(Some(too_many))).is_err());
    }
}
