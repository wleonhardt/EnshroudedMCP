//! Strict admission for one server player's read-only prompt target.
use serde_json::{Map, Value, json};

#[must_use]
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "player":{"type":"string","minLength":1,"maxLength":128}},"required":["player"]})
}

/// Refuses unknown fields and unbounded or ambiguous-looking selector text.
pub fn check(args: &Map<String, Value>) -> Result<String, String> {
    crate::args::check(&schema(), args)?;
    let selector = args["player"].as_str().ok_or("player must be a string")?;
    if selector.is_empty()
        || selector.len() > 128
        || selector.trim() != selector
        || selector.chars().any(char::is_control)
    {
        return Err(
            "player requires 1..128 UTF-8 bytes without surrounding whitespace or controls".into(),
        );
    }
    Ok(selector.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selector_is_required_bounded_and_read_only() {
        assert_eq!(
            check(json!({"player":"Researcher"}).as_object().unwrap()).unwrap(),
            "Researcher"
        );
        for value in [
            json!({}),
            json!({"player":7}),
            json!({"player":""}),
            json!({"player":" padded"}),
            json!({"player":"bad\nname"}),
            json!({"player":"界".repeat(43)}),
            json!({"player":"7","confirm":"server-write"}),
            json!({"player":"7","entity":1}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err());
        }
    }
}
