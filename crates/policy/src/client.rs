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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecipeSelector {
    Output(String),
    Id(u32),
}

pub fn recipe_schema() -> Value {
    let mut schema = item_schema(true);
    schema.as_object_mut().unwrap().remove("required");
    schema["properties"]["recipe_id"] = json!({"type":"integer","minimum":0,"maximum":u32::MAX});
    schema["oneOf"] = json!([{"required":["item"]},{"required":["recipe_id"]}]);
    schema
}

pub fn recipe(args: &Map<String, Value>) -> Result<(RecipeSelector, usize), String> {
    crate::args::check(&recipe_schema(), args)?;
    match (args.contains_key("item"), args.get("recipe_id")) {
        (true, None) => item(args, true).map(|(name, limit)| (RecipeSelector::Output(name), limit)),
        (false, Some(value)) => {
            let id = value
                .as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .ok_or("recipe_id must be an integer from 0 to 4294967295")?;
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(4) as usize;
            Ok((RecipeSelector::Id(id), limit))
        }
        _ => Err("provide exactly one item or recipe_id".into()),
    }
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

    #[test]
    fn recipe_selector_requires_exactly_one_bounded_native_identity_or_output() {
        for id in [0u32, 73, u32::MAX] {
            assert_eq!(
                recipe(json!({"recipe_id":id}).as_object().unwrap()).unwrap(),
                (RecipeSelector::Id(id), 4)
            );
        }
        assert_eq!(
            recipe(json!({"item":"wood","limit":8}).as_object().unwrap()).unwrap(),
            (RecipeSelector::Output("wood".into()), 8)
        );
        for value in [
            json!({}),
            json!({"item":"wood","recipe_id":1}),
            json!({"recipe_id":-1}),
            json!({"recipe_id":4294967296u64}),
            json!({"recipe_id":1.5}),
            json!({"recipe_id":"1"}),
            json!({"recipe_id":1,"limit":9}),
            json!({"recipe_id":1,"extra":true}),
        ] {
            assert!(recipe(value.as_object().unwrap()).is_err());
        }
    }
}
