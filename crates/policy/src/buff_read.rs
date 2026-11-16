//! Admission for one server entity's active buffs.
use serde_json::{Map, Value, json};

pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "entity":{"type":"integer","minimum":1,"maximum":u32::MAX},
        "limit":{"type":"integer","minimum":1,"maximum":64}},"required":["entity"]})
}

pub fn check(args: &Map<String, Value>) -> Result<(u32, usize), String> {
    crate::args::check(&schema(), args)?;
    Ok((
        args["entity"].as_u64().unwrap() as u32,
        args.get("limit").and_then(Value::as_u64).unwrap_or(32) as usize,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn one_session_entity_and_bounded_output_without_mutation_arguments() {
        assert_eq!(
            check(json!({"entity":u32::MAX}).as_object().unwrap()).unwrap(),
            (u32::MAX, 32)
        );
        assert_eq!(
            check(json!({"entity":1,"limit":64}).as_object().unwrap()).unwrap(),
            (1, 64)
        );
        for args in [
            json!({}),
            json!({"entity":0}),
            json!({"entity":1.0}),
            json!({"entity":4294967296u64}),
            json!({"entity":true}),
            json!({"entity":1,"limit":0}),
            json!({"entity":1,"limit":65}),
            json!({"entity":1,"limit":1.5}),
            json!({"entity":1,"confirm":"server-write"}),
            json!({"entity":1,"apply":7}),
            json!({"entity":1,"player":"someone"}),
        ] {
            assert!(check(args.as_object().unwrap()).is_err(), "{args}");
        }
    }
}
