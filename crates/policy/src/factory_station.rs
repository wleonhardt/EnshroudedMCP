//! Read-only admission for one session-local server factory station.
use serde_json::{Map, Value, json};

pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "entity":{"type":"integer","minimum":1,"maximum":u32::MAX}},"required":["entity"]})
}
pub fn check(args: &Map<String, Value>) -> Result<u32, String> {
    crate::args::check(&schema(), args)?;
    args["entity"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .filter(|n| *n != 0)
        .ok_or_else(|| "entity must be a nonzero server-session u32".into())
}
pub fn counts(running: usize, next: usize, requirements: usize) -> Result<(), String> {
    if running > 4 || next > 4 || requirements > 8 {
        return Err(
            "factory station exceeds four choices per recipe or eight requirement flags".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn server_entity_selector_and_observation_work_are_strictly_bounded() {
        assert_eq!(
            check(json!({"entity":u32::MAX}).as_object().unwrap()).unwrap(),
            u32::MAX
        );
        for value in [
            json!({}),
            json!({"entity":0}),
            json!({"entity":-1}),
            json!({"entity":4294967296u64}),
            json!({"entity":true}),
            json!({"entity":1.0}),
            json!({"entity":"7"}),
            json!({"entity":7,"confirm":"server-write"}),
            json!({"entity":7,"recipe_id":3}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err());
        }
        assert!(counts(4, 4, 8).is_ok());
        for (a, b, c) in [(5, 0, 0), (0, 5, 0), (0, 0, 9), (usize::MAX, 0, 0)] {
            assert!(counts(a, b, c).is_err());
        }
    }
}
