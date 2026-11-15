//! Admission for deliberate forward server clock requests, never undo.

use serde_json::{Map, Value, json};
use std::time::Duration;

pub const WAIT: Duration = Duration::from_secs(5);
pub const MAX_OPERATIONS: usize = 128;

#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub operation: String,
    pub target_minute: Option<u16>,
    pub expected_hours: Option<f64>,
    pub session: Option<String>,
    pub dry: bool,
    pub confirm: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub session: String,
    pub hours: f64,
    pub speed: f64,
    pub players: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Preview {
    pub target_hours: f32,
    pub forward_hours: f64,
}

#[must_use]
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["operation"],"properties":{
        "operation":{"type":"string","minLength":1,"maxLength":64},
        "target_minute":{"type":"integer","minimum":0,"maximum":1439},
        "expected_hours":{"type":"number","minimum":0,"exclusiveMaximum":24},
        "session":{"type":"string","minLength":1,"maxLength":128},
        "dry":{"type":"boolean","default":true},
        "confirm":{"type":"string","enum":[crate::write_gate::CONFIRM]}
    }})
}

/// Parses a status query or bounded target request; the write gate is separate.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let operation = args.get("operation").and_then(Value::as_str).unwrap_or("");
    if operation.is_empty()
        || operation.len() > 64
        || !operation
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return Err(
            "clock operation must contain 1 to 64 lowercase ASCII letters, digits, '-' or '_'"
                .into(),
        );
    }
    let target_minute = args
        .get("target_minute")
        .map(|v| {
            v.as_u64()
                .and_then(|n| u16::try_from(n).ok())
                .ok_or_else(|| "target_minute must be an integer from 0 to 1439".to_owned())
        })
        .transpose()?;
    let expected_hours = args.get("expected_hours").and_then(Value::as_f64);
    if expected_hours.is_some_and(|hours| !valid_hours(hours)) {
        return Err("expected_hours must be finite and at least 0 but below 24".into());
    }
    let session = args.get("session").and_then(Value::as_str);
    if session
        .is_some_and(|s| s.trim().is_empty() || s.len() > 128 || s.chars().any(char::is_control))
    {
        return Err("clock session must contain 1 to 128 bytes without control characters".into());
    }
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    let confirm = args.get("confirm").and_then(Value::as_str);
    if target_minute.is_none()
        && (!dry || expected_hours.is_some() || session.is_some() || confirm.is_some())
    {
        return Err("clock operation status cannot include write fields".into());
    }
    if target_minute.is_some() && !dry && (session.is_none() || expected_hours.is_none()) {
        return Err("real clock advance requires session and expected_hours".into());
    }
    Ok(Spec {
        operation: operation.into(),
        target_minute,
        expected_hours,
        session: session.map(str::to_owned),
        dry,
        confirm: confirm.map(str::to_owned),
    })
}

fn valid_hours(hours: f64) -> bool {
    hours.is_finite() && (0.0..24.0).contains(&hours)
}

/// Checks fresh game-thread observations, including the optional prior sample.
pub fn admit(spec: &Spec, current: &Context) -> Result<Preview, String> {
    if current.players != 0 {
        return Err("clock advance requires an empty SDK player-character roster".into());
    }
    if !valid_hours(current.hours)
        || !current.speed.is_finite()
        || !(0.0..=1.0).contains(&current.speed)
    {
        return Err("clock observation unavailable, invalid or accelerated".into());
    }
    if let Some(session) = &spec.session {
        crate::construction::check_session(session, &current.session)?;
    }
    if spec.expected_hours.is_some_and(|before| {
        !valid_hours(before) || (current.hours - before).rem_euclid(24.0) > 1.0 / 60.0
    }) {
        return Err("clock moved beyond the expected one-game-minute window".into());
    }
    let minute = spec
        .target_minute
        .ok_or("status query has no clock target")?;
    let target_hours = f32::from(minute) / 60.0;
    let forward_hours = (f64::from(target_hours) - current.hours).rem_euclid(24.0);
    if !(1.0..=12.0).contains(&forward_hours) {
        return Err(
            "clock target must be one to twelve game hours forward from the current observation"
                .into(),
        );
    }
    Ok(Preview {
        target_hours,
        forward_hours,
    })
}

#[must_use]
pub fn canonical_request(spec: &Spec) -> Value {
    json!({"target_minute":spec.target_minute,"session":spec.session,"expected_hours":spec.expected_hours})
}

pub fn check_replay(request: &Value, original: &Value) -> Result<(), String> {
    if request != original {
        return Err("clock identity already reserved for another request; never redispatch".into());
    }
    Ok(())
}

pub fn check_capacity(count: usize) -> Result<(), String> {
    if count >= MAX_OPERATIONS {
        return Err("clock operation capacity reached".into());
    }
    Ok(())
}

pub fn check_record_size(bytes: usize) -> Result<(), String> {
    if bytes > crate::write_gate::JOURNAL_RECORD_BYTES {
        return Err("clock record exceeds 4096 bytes".into());
    }
    Ok(())
}

pub fn check_elapsed(elapsed: Duration) -> Result<(), String> {
    if elapsed >= WAIT {
        return Err("clock request deadline expired before dispatch".into());
    }
    Ok(())
}

#[must_use]
pub fn readback_near(target: f32, observed: Option<&Context>) -> bool {
    observed.is_some_and(|now| {
        let target = f64::from(target);
        if !valid_hours(target) || !valid_hours(now.hours) {
            return false;
        }
        let distance = (now.hours - target).abs();
        distance.min(24.0 - distance) <= 1.0 / 60.0
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::time::Duration;

    fn parse(value: Value) -> Result<Spec, String> {
        check(value.as_object().unwrap())
    }

    fn context(hours: f64) -> Context {
        Context {
            session: "active".into(),
            hours,
            speed: 1.0,
            players: 0,
        }
    }

    #[test]
    fn status_is_read_only_and_target_defaults_to_dry() {
        let status = parse(json!({"operation":"fixture"})).unwrap();
        assert!(status.dry && status.target_minute.is_none());
        let preview = parse(json!({"operation":"fixture","target_minute":720})).unwrap();
        assert!(preview.dry && preview.target_minute == Some(720));
        for extra in [
            json!({"dry":false}),
            json!({"expected_hours":8}),
            json!({"session":"active"}),
            json!({"confirm":"server-write"}),
        ] {
            let mut value = json!({"operation":"fixture"});
            value
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            assert!(parse(value).is_err());
        }
    }

    #[test]
    fn identities_minutes_and_unknown_arguments_are_strict() {
        for operation in ["", "../escape", "Upper", "a/b", "a.b", "é", &"a".repeat(65)] {
            assert!(parse(json!({"operation":operation})).is_err());
        }
        for minute in [json!(-1), json!(1440), json!(1.0), json!(true), Value::Null] {
            assert!(parse(json!({"operation":"fixture","target_minute":minute})).is_err());
        }
        assert!(parse(json!({"operation":"fixture","unexpected":true})).is_err());
        assert!(parse(json!({"operation":"fixture","target_minute":1439})).is_ok());
    }

    #[test]
    fn real_target_requires_session_and_expected_clock() {
        for value in [
            json!({"operation":"fixture","target_minute":720,"dry":false}),
            json!({"operation":"fixture","target_minute":720,"dry":false,"session":"active"}),
            json!({"operation":"fixture","target_minute":720,"dry":false,"expected_hours":8}),
        ] {
            assert!(parse(value).is_err());
        }
        assert!(parse(json!({"operation":"fixture","target_minute":720,"dry":false,"session":"active","expected_hours":8})).is_ok());
        for session in ["", " ", "bad\n", &"x".repeat(129)] {
            assert!(
                parse(json!({"operation":"fixture","target_minute":720,"session":session}))
                    .is_err()
            );
        }
    }

    #[test]
    fn forward_window_includes_midnight_without_accepting_a_full_cycle() {
        let target = parse(json!({"operation":"fixture","target_minute":60})).unwrap();
        assert!((admit(&target, &context(23.0)).unwrap().forward_hours - 2.0).abs() < 1e-6);
        assert!(admit(&target, &context(1.0)).is_err());
        assert!(admit(&target, &context(1.1)).is_err());
        assert!(admit(&target, &context(0.5)).is_err());
        assert!(admit(&target, &context(13.0)).is_ok());
        assert!(admit(&target, &context(12.99)).is_err());
    }

    #[test]
    fn roster_speed_clock_and_stale_context_fail_closed() {
        let spec=parse(json!({"operation":"fixture","target_minute":720,"dry":false,"session":"active","expected_hours":8})).unwrap();
        assert!(admit(&spec, &context(8.0)).is_ok());
        for hours in [f64::NAN, f64::INFINITY, -1.0, 24.0, 8.1, 7.9] {
            assert!(admit(&spec, &context(hours)).is_err());
        }
        let mut value = context(8.0);
        value.players = 1;
        assert!(admit(&spec, &value).is_err());
        value.players = 0;
        for speed in [f64::NAN, f64::INFINITY, -1.0, 1.01] {
            value.speed = speed;
            assert!(admit(&spec, &value).is_err());
        }
        value.speed = 0.0;
        assert!(admit(&spec, &value).is_ok());
        value.session = "other".into();
        assert!(admit(&spec, &value).is_err());
    }

    #[test]
    fn status_cannot_be_admitted_as_a_target_and_budgets_are_bounded() {
        let status = parse(json!({"operation":"fixture"})).unwrap();
        assert!(admit(&status, &context(8.0)).is_err());
        assert!(check_capacity(MAX_OPERATIONS - 1).is_ok());
        assert!(check_capacity(MAX_OPERATIONS).is_err());
        assert!(check_record_size(4096).is_ok());
        assert!(check_record_size(4097).is_err());
        assert!(check_elapsed(Duration::from_secs(4)).is_ok());
        assert!(check_elapsed(Duration::from_secs(5)).is_err());
    }

    #[test]
    fn canonical_replay_ignores_gate_fields_but_refuses_changed_request_context() {
        let first=parse(json!({"operation":"fixture","target_minute":720,"session":"active","expected_hours":8})).unwrap();
        let real=parse(json!({"operation":"fixture","target_minute":720,"session":"active","expected_hours":8,"dry":false,"confirm":"server-write"})).unwrap();
        assert!(check_replay(&canonical_request(&first), &canonical_request(&real)).is_ok());
        for changed in [
            json!({"target_minute":780,"session":"active","expected_hours":8}),
            json!({"target_minute":720,"session":"other","expected_hours":8}),
            json!({"target_minute":720,"session":"active","expected_hours":9}),
        ] {
            assert!(check_replay(&canonical_request(&first), &changed).is_err());
        }
    }

    #[test]
    fn readback_is_only_a_near_target_observation_and_missing_is_not_zero() {
        assert!(readback_near(0.0, Some(&context(0.0))));
        assert!(readback_near(0.0, Some(&context(23.99))));
        assert!(!readback_near(0.0, None));
        assert!(!readback_near(0.0, Some(&context(23.9))));
        assert!(!readback_near(0.0, Some(&context(f64::NAN))));
    }
}
