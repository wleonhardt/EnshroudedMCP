//! Admission for reading or changing the single native enemy-pacification setting.

use serde_json::{Map, Value, json};

pub const RECORD_MAX_BYTES: usize = 4096;

/// Checks the bounded pacification journal record size.
///
/// # Errors
/// Refuses records larger than the journal headroom budget.
pub fn check_record_size(bytes: usize) -> Result<(), String> {
    if bytes > RECORD_MAX_BYTES {
        return Err("pacification journal record exceeds 4096 bytes".into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    pub enabled: Option<bool>,
    pub expected: Option<bool>,
    pub session: Option<String>,
    pub dry: bool,
    pub confirm: Option<String>,
}

#[must_use]
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "enabled":{"type":"boolean"},
        "expected":{"type":"boolean"},
        "session":{"type":"string","minLength":1,"maxLength":128},
        "dry":{"type":"boolean","default":true},
        "confirm":{"type":"string","enum":[crate::write_gate::CONFIRM]}
    }})
}

/// Parses the narrow request; a real change still requires the shared write gate.
///
/// # Errors
/// Refuses malformed fields, writes without current-value/session guards, or write fields on reads.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let enabled = args.get("enabled").and_then(Value::as_bool);
    let expected = args.get("expected").and_then(Value::as_bool);
    let session = args.get("session").and_then(Value::as_str);
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    let confirm = args.get("confirm").and_then(Value::as_str);
    if session.is_some_and(|value| {
        value.trim().is_empty() || value.len() > 128 || value.chars().any(char::is_control)
    }) {
        return Err("session must contain 1 to 128 bytes without control characters".into());
    }
    if enabled.is_none() && (!dry || expected.is_some() || session.is_some() || confirm.is_some()) {
        return Err("a read without enabled cannot include write fields".into());
    }
    if enabled.is_some() && !dry && (expected.is_none() || session.is_none()) {
        return Err("real pacification change requires expected and session".into());
    }
    Ok(Spec {
        enabled,
        expected,
        session: session.map(str::to_owned),
        dry,
        confirm: confirm.map(str::to_owned),
    })
}

/// Checks the previously observed boolean immediately before applying a change.
///
/// # Errors
/// Refuses a different current value; the adapter must also check the active world session.
pub fn check_current(expected: bool, current: bool) -> Result<(), String> {
    if expected != current {
        return Err("pacification no longer matches expected current value".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_budget_is_bounded() {
        assert!(check_record_size(RECORD_MAX_BYTES).is_ok());
        assert!(check_record_size(RECORD_MAX_BYTES + 1).is_err());
        assert!(check_record_size(usize::MAX).is_err());
    }

    fn parse(value: Value) -> Result<Spec, String> {
        check(value.as_object().unwrap())
    }

    #[test]
    fn empty_request_reads_and_setting_defaults_to_report_only() {
        let read = parse(json!({})).unwrap();
        assert!(read.dry);
        assert_eq!(read.enabled, None);
        for enabled in [false, true] {
            let preview = parse(json!({"enabled":enabled})).unwrap();
            assert!(preview.dry);
            assert_eq!(preview.enabled, Some(enabled));
        }
    }

    #[test]
    fn reads_refuse_write_fields() {
        for value in [
            json!({"dry":false}),
            json!({"expected":false}),
            json!({"session":"active"}),
            json!({"confirm":"server-write"}),
        ] {
            assert!(parse(value).is_err());
        }
        assert!(parse(json!({"dry":true})).is_ok());
    }

    #[test]
    fn real_change_requires_both_current_value_and_session() {
        for value in [
            json!({"enabled":true,"dry":false}),
            json!({"enabled":true,"dry":false,"expected":false}),
            json!({"enabled":true,"dry":false,"session":"active"}),
        ] {
            assert!(parse(value).is_err());
        }
        let spec = parse(json!({"enabled":true,"dry":false,"expected":false,
            "session":"active","confirm":"server-write"}))
        .unwrap();
        assert_eq!(spec.expected, Some(false));
        assert!(!spec.dry);
    }

    #[test]
    fn wrong_types_unknown_fields_and_bad_confirmation_refuse() {
        for value in [
            json!({"enabled":null}),
            json!({"enabled":1}),
            json!({"enabled":{}}),
            json!({"expected":"false"}),
            json!({"session":[]}),
            json!({"dry":"true"}),
            json!({"enabled":true,"confirm":"yes"}),
            json!({"enable":true}),
            json!({"enabled":true,"settings":{"other":true}}),
        ] {
            assert!(parse(value).is_err());
        }
    }

    #[test]
    fn session_is_byte_bounded_and_control_free() {
        for session in [
            "".to_owned(),
            " ".to_owned(),
            "a\nb".to_owned(),
            "é".repeat(65),
        ] {
            assert!(parse(json!({"enabled":true,"session":session})).is_err());
        }
        assert!(parse(json!({"enabled":true,"session":"x".repeat(128)})).is_ok());
    }

    #[test]
    fn current_value_guard_distinguishes_both_booleans() {
        for expected in [false, true] {
            assert!(check_current(expected, expected).is_ok());
            assert!(check_current(expected, !expected).is_err());
        }
    }
}
