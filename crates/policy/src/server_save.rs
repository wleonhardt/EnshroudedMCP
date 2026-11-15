//! Admission for a deliberate native server save request.
use serde_json::{Map, Value, json};
use std::time::Duration;
pub const WAIT: Duration = Duration::from_secs(5);
pub const MAX_OPERATIONS: usize = 64;
#[derive(Clone, Debug)]
pub struct Spec {
    pub operation: String,
    pub session: Option<String>,
    pub dry: bool,
    pub confirm: Option<String>,
}
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["operation"],"properties":{
        "operation":{"type":"string","minLength":1,"maxLength":64},
        "session":{"type":"string","minLength":1,"maxLength":128},
        "dry":{"type":"boolean","default":true},
        "confirm":{"type":"string","enum":[crate::write_gate::CONFIRM]}
    }})
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
            "save operation must contain 1 to 64 lowercase ASCII letters, digits, '-' or '_'"
                .into(),
        );
    }
    let session = args.get("session").and_then(Value::as_str);
    if session
        .is_some_and(|s| s.trim().is_empty() || s.len() > 128 || s.chars().any(char::is_control))
    {
        return Err("save session must contain 1 to 128 bytes without control characters".into());
    }
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    if !dry && session.is_none() {
        return Err("real save requires the preview session".into());
    }
    Ok(Spec {
        operation: operation.into(),
        session: session.map(str::to_owned),
        dry,
        confirm: args
            .get("confirm")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}
pub fn admit(spec: &Spec, session: &str, players: usize) -> Result<(), String> {
    if let Some(expected) = &spec.session {
        crate::construction::check_session(expected, session)?;
    }
    if players != 0 {
        return Err("save requires an empty SDK player-character roster; coordinate loading clients separately".into());
    }
    Ok(())
}
pub fn capacity(count: usize) -> Result<(), String> {
    if count >= MAX_OPERATIONS {
        Err("save operation capacity reached".into())
    } else {
        Ok(())
    }
}
pub fn elapsed(value: Duration) -> Result<(), String> {
    if value >= WAIT {
        Err("save request deadline expired before dispatch".into())
    } else {
        Ok(())
    }
}

/// Reserve intent and outcome headroom before any native request.
pub fn journal_headroom(current: u64) -> Result<(), String> {
    crate::construction::check_journal_size(current, 2 * crate::write_gate::JOURNAL_RECORD_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_save_identity_preview_session_and_empty_roster() {
        let dry = check(json!({"operation":"con"}).as_object().unwrap()).unwrap();
        assert!(dry.dry);
        assert!(admit(&dry, "active", 0).is_ok());
        for id in ["", "../x", "UPPER", "x y", "a/b", "a\\b"] {
            assert!(check(json!({"operation":id}).as_object().unwrap()).is_err());
        }
        assert!(check(json!({"operation":"x","unknown":0}).as_object().unwrap()).is_err());
        assert!(check(json!({"operation":"x","dry":false}).as_object().unwrap()).is_err());
        let real = check(
            json!({"operation":"x","dry":false,"session":"active"})
                .as_object()
                .unwrap(),
        )
        .unwrap();
        assert!(admit(&real, "active", 0).is_ok());
        assert!(admit(&real, "other", 0).is_err());
        assert!(admit(&real, "active", 1).is_err());
        assert!(capacity(63).is_ok());
        assert!(capacity(64).is_err());
        assert!(elapsed(WAIT).is_err());
        assert!(elapsed(Duration::ZERO).is_ok());
        let limit = crate::construction::JOURNAL_MAX_BYTES as u64;
        let headroom = (2 * crate::write_gate::JOURNAL_RECORD_BYTES) as u64;
        assert!(journal_headroom(limit - headroom).is_ok());
        assert!(journal_headroom(limit - headroom + 1).is_err());
    }
}
