//! Strict admission for the replay-safe administrative inventory grant.
use serde_json::{Map, Value, json};

pub const WAIT: std::time::Duration = std::time::Duration::from_secs(5);
pub const MAX_OPERATIONS: usize = 64;
#[derive(Debug, Clone)]
pub struct Spec {
    pub operation: String,
    pub preview: Option<String>,
    pub player: String,
    pub item: String,
    pub count: u32,
    pub dry: bool,
    pub confirm: Option<String>,
}
#[must_use]
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "operation":{"type":"string","minLength":1,"maxLength":64},
        "preview":{"type":"string","minLength":1,"maxLength":2048},
        "player":{"type":"string","minLength":1,"maxLength":128},
        "item":{"type":"string","minLength":1,"maxLength":128},
        "count":{"type":"integer","minimum":1,"maximum":100,"default":1},
        "dry":{"type":"boolean","default":true},
        "confirm":{"type":"string"}
    },"required":["operation","player","item"]})
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
    let operation = args["operation"].as_str().unwrap_or("");
    if operation.is_empty()
        || operation.len() > 64
        || !operation
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return Err(
            "grant operation must contain 1..64 lowercase ASCII letters, digits, '-' or '_'".into(),
        );
    }
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    let preview = args.get("preview").and_then(Value::as_str);
    if preview.is_some_and(|p| p.is_empty() || p.len() > 2048) || (!dry && preview.is_none()) {
        return Err("real grant requires the bounded current preview".into());
    }
    let count = match args.get("count") {
        None => 1,
        Some(value) => value
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| (1..=100).contains(n))
            .ok_or("count must be an integer from 1 to 100")?,
    };
    Ok(Spec {
        operation: operation.into(),
        preview: preview.map(str::to_owned),
        player: selector(&args["player"], "player")?,
        item: selector(&args["item"], "item")?,
        count,
        dry,
        confirm: args
            .get("confirm")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}
pub fn request(spec: &Spec) -> Value {
    json!({"player":spec.player,"item":spec.item,"count":spec.count})
}
pub fn preview(spec: &Spec, session: &str, resolved: &Value) -> String {
    json!({"operation":spec.operation,"request":request(spec),"session":session,"resolved":resolved}).to_string()
}
pub fn check_preview(expected: Option<&str>, current: &str) -> Result<(), String> {
    if expected == Some(current) {
        Ok(())
    } else {
        Err("grant preview changed; resolve the current actor, item and session again".into())
    }
}
pub fn capacity(count: usize) -> Result<(), String> {
    if count < MAX_OPERATIONS {
        Ok(())
    } else {
        Err("grant operation capacity reached".into())
    }
}
pub fn elapsed(age: std::time::Duration) -> Result<(), String> {
    if age < WAIT {
        Ok(())
    } else {
        Err("grant deadline expired before native submission".into())
    }
}
pub fn journal_headroom(current: u64) -> Result<(), String> {
    crate::construction::check_journal_size(current, 2 * crate::write_gate::JOURNAL_RECORD_BYTES)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn args() -> Map<String, Value> {
        json!({"operation":"fixture","player":"actor","item":"fixture_item"})
            .as_object()
            .unwrap()
            .clone()
    }
    #[test]
    fn omitted_defaults_and_positive_count_boundaries_are_preserved() {
        let mut input = args();
        let request = check(&input).unwrap();
        assert_eq!(request.count, 1);
        assert!(request.dry);
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
    #[test]
    fn caller_identity_and_dry_default_are_required_for_safe_grant_migration() {
        assert!(
            check(
                json!({"player":"actor","item":"fixture_item"})
                    .as_object()
                    .unwrap()
            )
            .is_err()
        );
        let request = check(
            json!({"operation":"once","player":"actor","item":"fixture_item"})
                .as_object()
                .unwrap(),
        )
        .unwrap();
        assert!(request.dry);
        assert!(
            check(
                json!({"operation":"once","player":"actor","item":"fixture_item","dry":false})
                    .as_object()
                    .unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn operation_preview_capacity_and_deadline_admission_are_bounded() {
        for operation in ["", "../x", "UPPER", "x y", "a/b", &"a".repeat(65)] {
            let mut input = args();
            input.insert("operation".into(), json!(operation));
            assert!(check(&input).is_err());
        }
        assert!(capacity(63).is_ok());
        assert!(capacity(64).is_err());
        assert!(elapsed(WAIT).is_err());
        assert!(elapsed(std::time::Duration::ZERO).is_ok());
        let limit = crate::construction::JOURNAL_MAX_BYTES as u64;
        let headroom = (2 * crate::write_gate::JOURNAL_RECORD_BYTES) as u64;
        assert!(journal_headroom(limit - headroom).is_ok());
        assert!(journal_headroom(limit - headroom + 1).is_err());
    }
    #[test]
    fn preview_binds_payload_operation_actor_item_and_session() {
        let spec = check(&args()).unwrap();
        let resolved = json!({"actor":7,"item_id":9});
        let token = preview(&spec, "active", &resolved);
        assert!(check_preview(Some(&token), &token).is_ok());
        for changed in [
            preview(&spec, "other", &resolved),
            preview(&spec, "active", &json!({"actor":8,"item_id":9})),
            preview(&spec, "active", &json!({"actor":7,"item_id":10})),
        ] {
            assert!(check_preview(Some(&token), &changed).is_err());
        }
        let mut changed = spec.clone();
        changed.count += 1;
        assert_ne!(preview(&changed, "active", &resolved), token);
        changed = spec.clone();
        changed.operation = "other".into();
        assert_ne!(preview(&changed, "active", &resolved), token);
    }
}
