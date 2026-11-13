//! Client research admission. No native UI or game dependency.
use serde_json::{Map, Value, json};

pub const HISTORY: usize = 64;
pub const TEXT: usize = 256;

pub fn item_schema(recipe: bool) -> Value {
    let mut schema =
        json!({"type":"object","properties":{"item":{"type":"string"}},"required":["item"]});
    if recipe {
        schema["properties"]["limit"] = json!({"type":"integer","minimum":1,"maximum":8});
    }
    schema
}

pub fn item(args: &Map<String, Value>, recipe: bool) -> Result<(String, usize), String> {
    crate::args::check(&item_schema(recipe), args)?;
    let name = args["item"]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 128 && !s.chars().any(char::is_control))
        .ok_or("item must contain 1..128 UTF-8 bytes without control characters")?;
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(4) as usize;
    Ok((name.to_owned(), limit))
}

pub fn events_schema() -> Value {
    json!({"type":"object","properties":{"after":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":32}}})
}

pub fn events(args: &Map<String, Value>) -> Result<(u64, usize), String> {
    crate::args::check(&events_schema(), args)?;
    let after = match args.get("after") {
        None => 0,
        Some(v) => v.as_u64().ok_or("after must be a nonnegative u64")?,
    };
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(32) as usize;
    Ok((after, limit))
}

pub fn text(s: &str) -> String {
    let mut end = s.len().min(TEXT);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selectors_refuse_unknown_fields_and_unbounded_text() {
        assert_eq!(
            item(json!({"item":"wood"}).as_object().unwrap(), false)
                .unwrap()
                .0,
            "wood"
        );
        for value in [
            json!({"item":""}),
            json!({"item":"x\n"}),
            json!({"item":"x","extra":1}),
            json!({"item":"x","limit":9}),
            json!({"item":"x".repeat(129)}),
        ] {
            assert!(item(value.as_object().unwrap(), true).is_err());
        }
    }
    #[test]
    fn event_cursor_limits_and_unicode_output_are_bounded() {
        assert_eq!(events(json!({}).as_object().unwrap()).unwrap(), (0, 32));
        for value in [
            json!({"after":-1}),
            json!({"after":1.5}),
            json!({"limit":0}),
            json!({"limit":33}),
            json!({"clear":true}),
        ] {
            assert!(events(value.as_object().unwrap()).is_err());
        }
        let value = text(&"界".repeat(100));
        assert!(value.len() <= TEXT);
        assert_eq!(value.chars().count(), 85);
    }
}
