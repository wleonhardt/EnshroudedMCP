//! Strict admission for the legacy administrative inventory grant.
use serde_json::{Map, Value, json};

#[derive(Debug)]
pub struct Spec {
    pub player: String,
    pub item: String,
    pub count: u32,
    pub dry: bool,
    pub confirm: Option<String>,
}
#[must_use]
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "player":{"type":"string","minLength":1,"maxLength":128},
        "item":{"type":"string","minLength":1,"maxLength":128},
        "count":{"type":"integer","minimum":1,"maximum":100,"default":1},
        "dry":{"type":"boolean","default":false},
        "confirm":{"type":"string"}
    },"required":["player","item"]})
}
fn selector(value: &Value, field: &str) -> Result<String, String> {
    value
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 128 && !s.chars().any(char::is_control))
        .map(str::to_owned)
        .ok_or_else(|| {
            format!("{field} must contain 1 to 128 UTF-8 bytes without control characters")
        })
}
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let count = match args.get("count") {
        None => 1,
        Some(value) => value
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| (1..=100).contains(n))
            .ok_or("count must be an integer from 1 to 100")?,
    };
    Ok(Spec {
        player: selector(&args["player"], "player")?,
        item: selector(&args["item"], "item")?,
        count,
        dry: args.get("dry").and_then(Value::as_bool).unwrap_or(false),
        confirm: args
            .get("confirm")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn args() -> Map<String, Value> {
        json!({"player":"actor","item":"fixture_item"})
            .as_object()
            .unwrap()
            .clone()
    }
    #[test]
    fn omitted_defaults_and_positive_count_boundaries_are_preserved() {
        let mut input = args();
        let request = check(&input).unwrap();
        assert_eq!(request.count, 1);
        assert!(!request.dry);
        assert!(request.confirm.is_none());
        for count in [1, 100] {
            input.insert("count".into(), json!(count));
            assert_eq!(check(&input).unwrap().count, count);
        }
        input.insert("dry".into(), json!(true));
        assert!(check(&input).unwrap().dry);
    }
    #[test]
    fn malformed_counts_never_become_the_default_quantity() {
        for count in [
            json!(-1),
            json!(0),
            json!(101),
            json!(1.5),
            json!("1"),
            json!(true),
            json!(null),
            json!(u64::MAX),
        ] {
            let mut input = args();
            input.insert("count".into(), count);
            assert!(check(&input).is_err());
        }
    }
    #[test]
    fn selectors_types_required_fields_and_unknown_arguments_are_strict() {
        for key in ["player", "item"] {
            for text in [
                "".into(),
                "  ".into(),
                "x\n".into(),
                "x".repeat(129),
                "é".repeat(65),
            ] {
                let mut input = args();
                input.insert(key.into(), json!(text));
                assert!(check(&input).is_err());
            }
            let mut input = args();
            input.insert(key.into(), json!("é".repeat(64)));
            assert!(check(&input).is_ok());
            input.remove(key);
            assert!(check(&input).is_err());
        }
        for (key, value) in [
            ("unknown", json!(1)),
            ("dry", json!("true")),
            ("confirm", json!(true)),
            ("player", json!(7)),
            ("item", json!(null)),
        ] {
            let mut input = args();
            input.insert(key.into(), value);
            assert!(check(&input).is_err());
        }
    }
}
