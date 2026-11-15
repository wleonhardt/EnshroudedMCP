//! Admission for a deliberate native dedicated-server stop request.
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
            "shutdown operation must contain 1 to 64 lowercase ASCII letters, digits, '-' or '_'"
                .into(),
        );
    }
    let session = args.get("session").and_then(Value::as_str);
    if session
        .is_some_and(|s| s.trim().is_empty() || s.len() > 128 || s.chars().any(char::is_control))
    {
        return Err(
            "shutdown session must contain 1 to 128 bytes without control characters".into(),
        );
    }
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    if !dry && session.is_none() {
        return Err("real shutdown requires the preview session".into());
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
        return Err("shutdown requires an empty SDK player-character roster; coordinate loading clients separately".into());
    }
    Ok(())
}
pub fn capacity(count: usize) -> Result<(), String> {
    if count >= MAX_OPERATIONS {
        Err("shutdown operation capacity reached".into())
    } else {
        Ok(())
    }
}
pub fn elapsed(value: Duration) -> Result<(), String> {
    if value >= WAIT {
        Err("shutdown request deadline expired before dispatch".into())
    } else {
        Ok(())
    }
}

pub fn attempt(session: &str, attempted: Option<&str>) -> Result<(), String> {
    if attempted == Some(session) {
        Err("a shutdown was already attempted in this session; reconcile without retry".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn strict_ids_preview_and_real_session_admission() {
        let preview = check(json!({"operation":"con"}).as_object().unwrap()).unwrap();
        assert!(preview.dry);
        assert_eq!(admit(&preview, "active", 0), Ok(()));
        let real=check(json!({"operation":"shutdown-1","session":"active","dry":false,"confirm":"server-write"}).as_object().unwrap()).unwrap();
        assert!(admit(&real, "other", 0).is_err());
        assert!(admit(&real, "active", 1).is_err());
        for value in [
            json!({}),
            json!({"operation":"../bad"}),
            json!({"operation":"A"}),
            json!({"operation":"x".repeat(65)}),
            json!({"operation":"x","dry":false}),
            json!({"operation":"x","session":"\n"}),
            json!({"operation":"x","extra":1}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err());
        }
    }
    #[test]
    fn workload_and_deadline_limits_are_explicit() {
        assert!(capacity(63).is_ok());
        assert!(capacity(64).is_err());
        assert!(elapsed(std::time::Duration::from_secs(4)).is_ok());
        assert!(elapsed(WAIT).is_err());
    }
}
