//! Admission for one bounded reposition request. No command completion or arrival is implied.

use serde_json::{Map, Value, json};

pub const MAX_PREVIEW_BYTES: usize = 1024;
pub const MAX_OPERATIONS: usize = 64;
pub const MAX_RECORD_BYTES: usize = 4096;

pub fn status_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "operation":{"type":"string","minLength":1,"maxLength":64}},"required":["operation"]})
}

fn check_operation(operation: &str) -> Result<(), String> {
    if operation.is_empty()
        || operation.len() > 64
        || !operation
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err(
            "operation must contain 1 to 64 ASCII letters, digits, underscores or hyphens".into(),
        );
    }
    Ok(())
}

pub fn check_status(args: &Map<String, Value>) -> Result<String, String> {
    crate::args::check(&status_schema(), args)?;
    let operation = args["operation"].as_str().unwrap();
    check_operation(operation)?;
    Ok(operation.to_owned())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub operation: String,
    pub player: String,
    pub target: [f64; 3],
    pub dry: bool,
    pub confirm: Option<String>,
    pub preview: Option<String>,
}

#[must_use]
pub fn schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "operation":{"type":"string","minLength":1,"maxLength":64},
        "player":{"type":"string","minLength":1,"maxLength":128},
        "x":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE},
        "y":{"type":"number","minimum":0,"maximum":crate::bounds::HEIGHT},
        "z":{"type":"number","minimum":0,"maximum":crate::bounds::EDGE},
        "dry":{"type":"boolean","default":true},
        "confirm":{"type":"string","enum":[crate::write_gate::CONFIRM]},
        "preview":{"type":"string","minLength":1,"maxLength":MAX_PREVIEW_BYTES}
    },"required":["operation","player","x","y","z"]})
}

fn text_ok(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

/// Parses admission arguments; the adapter must independently apply the write gate.
/// # Errors
/// Refuses unknown, malformed or excessive arguments and missing real-call previews.
pub fn check(args: &Map<String, Value>) -> Result<Spec, String> {
    crate::args::check(&schema(), args)?;
    let operation = args["operation"].as_str().unwrap();
    check_operation(operation)?;
    let player = args["player"].as_str().unwrap();
    if !text_ok(player, 128) {
        return Err("player must contain 1 to 128 bytes without control characters".into());
    }
    let target = crate::args::point(args, "x", "y", "z");
    if !crate::bounds::in_world(target) {
        return Err("reposition target must be finite and inside world bounds".into());
    }
    let dry = args.get("dry").and_then(Value::as_bool).unwrap_or(true);
    let preview = args.get("preview").and_then(Value::as_str);
    if preview.is_some_and(|token| !text_ok(token, MAX_PREVIEW_BYTES))
        || (!dry && preview.is_none())
    {
        return Err("real reposition requires a nonempty preview of at most 1024 bytes".into());
    }
    Ok(Spec {
        operation: operation.into(),
        player: player.into(),
        target,
        dry,
        confirm: args
            .get("confirm")
            .and_then(Value::as_str)
            .map(str::to_owned),
        preview: preview.map(str::to_owned),
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActorContext {
    pub session: String,
    pub actor_generation: u64,
    pub entity: u32,
    pub account: u64,
    pub position: [f64; 3],
}

#[derive(Clone, Debug, Default)]
pub struct ObservationFacts {
    pub session: Option<String>,
    pub generation: u64,
    pub account: Option<u64>,
    pub entity: Option<u32>,
    pub health: Option<i32>,
    pub position: Option<[f64; 3]>,
}

pub fn observation_scope(
    expected: &ActorContext,
    session: Option<&str>,
    generation: u64,
) -> Option<&'static str> {
    match session {
        None => Some("inactive_session"),
        Some(session) if session != expected.session => Some("stale_session"),
        _ if generation != expected.actor_generation => Some("actor_context_changed"),
        _ => None,
    }
}

pub fn observe_position(
    expected: &ActorContext,
    target: [f64; 3],
    facts: ObservationFacts,
) -> Value {
    let measured = facts
        .position
        .map(|position| {
            let distance = position
                .into_iter()
                .zip(target)
                .fold(0.0_f64, |distance, (a, b)| distance.hypot(a - b));
            (position, distance)
        })
        .filter(|(position, distance)| {
            position.iter().all(|value| value.is_finite()) && distance.is_finite()
        });
    let state = observation_scope(expected, facts.session.as_deref(), facts.generation)
        .unwrap_or_else(|| {
            if facts.account.is_none() {
                "actor_not_observed"
            } else if facts.account != Some(expected.account)
                || facts.entity != Some(expected.entity)
            {
                "actor_context_changed"
            } else if !facts.health.is_some_and(|value| value > 0) || measured.is_none() {
                "unavailable"
            } else {
                "observed"
            }
        });
    let measured = measured.filter(|_| state == "observed");
    let position = measured.map(|(position, _)| position);
    let distance = measured.map(|(_, distance)| distance);
    json!({"state":state,"current_position_m":position,"distance_to_requested_m":distance,
        "health_current":facts.health.filter(|_|state=="observed")})
}

/// # Errors
/// Refuses nonfinite/outside-world points or a requested displacement exceeding 16 metres.
pub fn check_distance(origin: [f64; 3], target: [f64; 3]) -> Result<(), String> {
    if !crate::bounds::in_world(origin) || !crate::bounds::in_world(target) {
        return Err("reposition points must be finite and inside world bounds".into());
    }
    if origin
        .into_iter()
        .zip(target)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        > 256.0
    {
        return Err("requested reposition exceeds 16 metres".into());
    }
    Ok(())
}

/// Exact context serialization, not an authentication credential or arrival receipt.
/// # Errors
/// Refuses invalid actor identity, excessive context or an inadmissible requested point.
pub fn preview_token(context: &ActorContext, target: [f64; 3]) -> Result<String, String> {
    if !text_ok(&context.session, 128) || context.entity == 0 || context.account == 0 {
        return Err("reposition actor context is unavailable".into());
    }
    check_distance(context.position, target)?;
    let token = json!({"session":context.session,"actor_generation":context.actor_generation,
        "entity":context.entity,"account":context.account.to_string(),"position":context.position,"target":target}).to_string();
    if token.len() > MAX_PREVIEW_BYTES {
        return Err("reposition preview exceeds 1024 bytes".into());
    }
    Ok(token)
}

/// # Errors
/// Refuses any change to the previewed actor, world session, position or requested point.
pub fn check_preview(token: &str, context: &ActorContext, target: [f64; 3]) -> Result<(), String> {
    if token.len() > MAX_PREVIEW_BYTES || token != preview_token(context, target)? {
        return Err("reposition preview no longer matches current actor context".into());
    }
    Ok(())
}

#[must_use]
pub fn canonical_request(spec: &Spec) -> Value {
    json!({"operation":spec.operation,"player":spec.player,"target":spec.target,"preview":spec.preview})
}

/// # Errors
/// Refuses reuse of an operation identifier for another canonical request.
pub fn same_request(existing: &Value, requested: &Value) -> Result<(), String> {
    if existing != requested {
        return Err("operation already belongs to a different reposition request".into());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub struct ActorFacts {
    pub connected_players: usize,
    pub matching_actors: usize,
    pub health: Option<i32>,
}

/// # Errors
/// Refuses ambiguous/disconnected actors or unavailable/nonpositive health. No role is inferred.
pub fn check_actor(facts: ActorFacts) -> Result<(), String> {
    if facts.connected_players != 1 || facts.matching_actors != 1 {
        return Err("reposition requires exactly one connected matching actor".into());
    }
    if !facts.health.is_some_and(|health| health > 0) {
        return Err("reposition requires observed positive actor health".into());
    }
    Ok(())
}

/// # Errors
/// Refuses another operation once the bounded in-memory ledger is full.
pub fn check_capacity(count: usize) -> Result<(), String> {
    if count >= MAX_OPERATIONS {
        return Err("reposition operation capacity reached".into());
    }
    Ok(())
}

/// # Errors
/// Refuses an oversized durable record before its write.
pub fn check_record_size(bytes: usize) -> Result<(), String> {
    if bytes > MAX_RECORD_BYTES {
        return Err("reposition record exceeds 4096 bytes".into());
    }
    Ok(())
}

/// # Errors
/// Refuses admission without space for both intent and dispatch-outcome records.
pub fn check_journal_headroom(current: u64) -> Result<(), String> {
    crate::construction::check_journal_size(current, 2 * MAX_RECORD_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_admits_only_a_bounded_existing_operation_selector() {
        assert_eq!(
            check_status(json!({"operation":"Platform_01"}).as_object().unwrap()).unwrap(),
            "Platform_01"
        );
        for value in [
            json!({}),
            json!({"operation":"../other"}),
            json!({"operation":true}),
            json!({"operation":"x".repeat(65)}),
            json!({"operation":"once","confirm":"server-write"}),
            json!({"operation":"once","player":"actor"}),
            json!({"operation":"once","dry":true}),
        ] {
            assert!(check_status(value.as_object().unwrap()).is_err());
        }
    }

    #[test]
    fn observation_preserves_distance_without_an_arrival_threshold() {
        let expected = ActorContext {
            session: "s".into(),
            actor_generation: 4,
            entity: 7,
            account: 9,
            position: [100., 20., 100.],
        };
        let facts = ObservationFacts {
            session: Some("s".into()),
            generation: 4,
            account: Some(9),
            entity: Some(7),
            health: Some(150),
            position: Some([120., 20., 100.]),
        };
        let result = observe_position(&expected, [101., 20., 100.], facts.clone());
        assert_eq!(result["state"], "observed");
        assert_eq!(result["distance_to_requested_m"], 19.0);
        let exact = observe_position(&expected, [120., 20., 100.], facts);
        assert_eq!(exact["distance_to_requested_m"], 0.0);
        assert!(exact.get("arrival_verified").is_none());
        // Write admission bounds do not turn a finite read into unavailable data.
        for position in [[-1., 20., 100.], [100., 1025., 100.]] {
            let facts = ObservationFacts {
                session: Some("s".into()),
                generation: 4,
                account: Some(9),
                entity: Some(7),
                health: Some(150),
                position: Some(position),
            };
            let observed = observe_position(&expected, [101., 20., 100.], facts);
            assert_eq!(observed["state"], "observed");
            assert_eq!(observed["current_position_m"], json!(position));
        }
    }

    #[test]
    fn changed_context_or_missing_actor_data_never_exposes_a_current_position() {
        let expected = ActorContext {
            session: "s".into(),
            actor_generation: 4,
            entity: 7,
            account: 9,
            position: [100., 20., 100.],
        };
        let good = ObservationFacts {
            session: Some("s".into()),
            generation: 4,
            account: Some(9),
            entity: Some(7),
            health: Some(150),
            position: Some([100., 20., 100.]),
        };
        let mut cases = Vec::new();
        let mut value = good.clone();
        value.session = None;
        cases.push((value, "inactive_session"));
        let mut value = good.clone();
        value.session = Some("other".into());
        cases.push((value, "stale_session"));
        let mut value = good.clone();
        value.generation += 1;
        cases.push((value, "actor_context_changed"));
        let mut value = good.clone();
        value.account = None;
        value.entity = None;
        cases.push((value, "actor_not_observed"));
        let mut value = good.clone();
        value.account = Some(10);
        cases.push((value, "actor_context_changed"));
        let mut value = good.clone();
        value.entity = Some(8);
        cases.push((value, "actor_context_changed"));
        for health in [None, Some(0), Some(-1)] {
            let mut value = good.clone();
            value.health = health;
            cases.push((value, "unavailable"));
        }
        for position in [
            None,
            Some([f64::NAN, 20., 100.]),
            Some([f64::INFINITY, 20., 100.]),
        ] {
            let mut value = good.clone();
            value.position = position;
            cases.push((value, "unavailable"));
        }
        for (facts, state) in cases {
            let result = observe_position(&expected, [101., 20., 100.], facts);
            assert_eq!(result["state"], state);
            assert!(result["current_position_m"].is_null());
            assert!(result["distance_to_requested_m"].is_null());
        }
    }
    use serde_json::json;

    fn request() -> serde_json::Map<String, serde_json::Value> {
        json!({"operation":"move-1","player":"actor","x":110,"y":20,"z":100})
            .as_object()
            .unwrap()
            .clone()
    }

    fn actor() -> ActorContext {
        ActorContext {
            session: "boot-1".into(),
            actor_generation: 1,
            entity: 7,
            account: 9,
            position: [100.0, 20.0, 100.0],
        }
    }

    #[test]
    fn strict_request_defaults_to_report_only() {
        assert!(check(&request()).unwrap().dry);
        for (key, value) in [
            ("drу", json!(false)),
            ("dry", json!("false")),
            ("x", json!(-1)),
            ("player", json!(" ")),
            ("operation", json!("../move")),
        ] {
            let mut args = request();
            args.insert(key.into(), value);
            assert!(check(&args).is_err(), "{key}");
        }
        let mut args = request();
        args.insert("dry".into(), json!(false));
        assert!(check(&args).is_err());
        args.insert("preview".into(), json!("opaque"));
        assert!(check(&args).is_ok());
        args.insert("preview".into(), json!("x".repeat(MAX_PREVIEW_BYTES + 1)));
        assert!(check(&args).is_err());
    }

    #[test]
    fn radial_distance_and_finite_world_bounds_are_enforced() {
        assert!(check_distance([100.0, 20.0, 100.0], [116.0, 20.0, 100.0]).is_ok());
        assert!(check_distance([100.0, 20.0, 100.0], [112.0, 20.0, 112.0]).is_err());
        for p in [
            [f64::NAN, 20.0, 100.0],
            [100.0, f64::INFINITY, 100.0],
            [-1.0, 20.0, 100.0],
            [100.0, 1025.0, 100.0],
        ] {
            assert!(check_distance(p, [100.0, 20.0, 100.0]).is_err());
            assert!(check_distance([100.0, 20.0, 100.0], p).is_err());
        }
    }

    #[test]
    fn selector_limits_count_bytes_and_real_write_still_needs_gate() {
        let mut args = request();
        args.insert("player".into(), json!("é".repeat(64)));
        args.insert("operation".into(), json!("a".repeat(64)));
        assert!(check(&args).is_ok());
        args.insert("player".into(), json!("é".repeat(65)));
        assert!(check(&args).is_err());
        args = request();
        args.insert("operation".into(), json!("a".repeat(65)));
        assert!(check(&args).is_err());
        args = request();
        args.insert("dry".into(), json!(false));
        args.insert("preview".into(), json!("opaque"));
        let spec = check(&args).unwrap();
        assert!(
            crate::write_gate::check(
                spec.dry,
                true,
                Some(std::time::Duration::ZERO),
                spec.confirm.as_deref()
            )
            .is_err()
        );
        args.insert("confirm".into(), json!("wrong"));
        assert!(check(&args).is_err());
        args.insert("confirm".into(), json!("server-write"));
        args.insert("preview".into(), json!("\n"));
        assert!(check(&args).is_err());
    }

    #[test]
    fn preview_binds_every_actor_field_and_requested_point() {
        let a = actor();
        let target = [110.0, 20.0, 100.0];
        let token = preview_token(&a, target).unwrap();
        assert!(token.len() <= MAX_PREVIEW_BYTES);
        assert!(check_preview(&token, &a, target).is_ok());
        let mut contexts = Vec::new();
        let mut b = a.clone();
        b.session = "boot-2".into();
        contexts.push(b);
        let mut b = a.clone();
        b.actor_generation += 1;
        contexts.push(b);
        let mut b = a.clone();
        b.entity += 1;
        contexts.push(b);
        let mut b = a.clone();
        b.account += 1;
        contexts.push(b);
        let mut b = a.clone();
        b.position[0] += 0.01;
        contexts.push(b);
        for b in contexts {
            assert!(check_preview(&token, &b, target).is_err());
        }
        assert!(check_preview(&token, &a, [111.0, 20.0, 100.0]).is_err());
    }

    #[test]
    fn replay_ignores_transport_mode_but_preserves_intent() {
        let mut a = check(&request()).unwrap();
        a.preview = Some("opaque".into());
        let first = canonical_request(&a);
        a.dry = false;
        a.confirm = Some("server-write".into());
        assert!(same_request(&first, &canonical_request(&a)).is_ok());
        a.target[0] += 1.0;
        assert!(same_request(&first, &canonical_request(&a)).is_err());
    }

    #[test]
    fn actor_and_storage_bounds_fail_closed() {
        assert!(
            check_actor(ActorFacts {
                connected_players: 1,
                matching_actors: 1,
                health: Some(1)
            })
            .is_ok()
        );
        for facts in [
            ActorFacts {
                connected_players: 2,
                matching_actors: 1,
                health: Some(1),
            },
            ActorFacts {
                connected_players: 1,
                matching_actors: 0,
                health: Some(1),
            },
            ActorFacts {
                connected_players: 1,
                matching_actors: 1,
                health: None,
            },
            ActorFacts {
                connected_players: 1,
                matching_actors: 1,
                health: Some(0),
            },
        ] {
            assert!(check_actor(facts).is_err());
        }
        assert!(check_capacity(63).is_ok());
        assert!(check_capacity(64).is_err());
        assert!(check_record_size(4096).is_ok());
        assert!(check_record_size(4097).is_err());
        let max = crate::construction::JOURNAL_MAX_BYTES as u64;
        assert!(check_journal_headroom(max - 8192).is_ok());
        assert!(check_journal_headroom(max - 8191).is_err());
        assert!(check_journal_headroom(u64::MAX).is_err());
    }
}
