//! Strict current-player selection and bounded active-buff output.
use serde_json::{Map, Value, json};

pub fn schema() -> Value {
    let mut value = crate::interaction_target::schema();
    value["properties"]["limit"] = crate::buff_read::schema()["properties"]["limit"].clone();
    value
}

pub fn check(args: &Map<String, Value>) -> Result<(String, usize), String> {
    crate::args::check(&schema(), args)?;
    let selector =
        crate::interaction_target::check(json!({"player":args["player"]}).as_object().unwrap())?;
    Ok((
        selector,
        args.get("limit").and_then(Value::as_u64).unwrap_or(32) as usize,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_selection_is_required_bounded_and_read_only() {
        assert_eq!(
            check(json!({"player":"Researcher"}).as_object().unwrap()).unwrap(),
            ("Researcher".into(), 32)
        );
        for args in [
            json!({}),
            json!({"player":7}),
            json!({"player":""}),
            json!({"player":" padded"}),
            json!({"player":"bad\nname"}),
            json!({"player":"界".repeat(43)}),
            json!({"player":"7","entity":17}),
            json!({"player":"7","dry":true}),
            json!({"player":"7","confirm":"server-write"}),
            json!({"player":"7","apply":1}),
        ] {
            assert!(check(args.as_object().unwrap()).is_err(), "{args}");
        }
    }
    #[test]
    fn output_limit_defaults_and_bounds_are_strict() {
        for limit in [1, 64] {
            assert_eq!(
                check(json!({"player":"7","limit":limit}).as_object().unwrap())
                    .unwrap()
                    .1,
                limit
            );
        }
        for limit in [
            json!(0),
            json!(65),
            json!(1.5),
            json!(true),
            json!("1"),
            Value::Null,
        ] {
            assert!(check(json!({"player":"7","limit":limit}).as_object().unwrap()).is_err());
        }
    }
}
