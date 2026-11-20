//! Strict read-only player admission and independent shroud observations.
use serde_json::{Value, json};

pub use crate::interaction_target::{check, schema};

pub fn require_world(key: Option<&str>) -> Result<(), String> {
    if key.is_none_or(str::is_empty) {
        return Err("no active server world key is available".into());
    }
    Ok(())
}

pub fn observation(
    entity: u32,
    timer: Option<(i32, i32)>,
    inside: Option<bool>,
) -> Result<Value, String> {
    if entity == 0 {
        return Err("native shroud observation has a zero server entity".into());
    }
    let state = match (timer.is_some(), inside.is_some()) {
        (true, true) => "observed",
        (false, false) => "unavailable",
        _ => "partial",
    };
    Ok(json!({
        "source":"published_world_shroud","player_entity":entity,"state":state,
        "in_shroud":inside,"timer_seconds":timer.map(|(current,max)|json!({"current":current,"max":max})),
        "units":"SDK integer seconds; UI correspondence unverified",
        "missing_semantics":"absent, unreadable or unsupported; no safe-time or outside-shroud default",
        "entity_scope":"server character; session only, never a client or durable selector",
        "world_session":"unavailable","atomic_snapshot":false,
        "world_mutated":false,"write_authorized":false,"current_client_verified":false,
        "rendered_ui_verified":false,"traversal_safety_verified":false,
        "damage_deadline_verified":false,"refill_requested":false
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_empty_world_context_refuses_observation() {
        assert!(require_world(None).is_err());
        assert!(require_world(Some("")).is_err());
        assert!(require_world(Some("fixture-shroud-world")).is_ok());
    }

    #[test]
    fn player_selector_is_strict_bounded_and_read_only() {
        assert_eq!(
            check(json!({"player":"Researcher"}).as_object().unwrap()).unwrap(),
            "Researcher"
        );
        for value in [
            json!({}),
            json!({"player":17}),
            json!({"player":""}),
            json!({"player":" padded"}),
            json!({"player":"bad\nname"}),
            json!({"player":"界".repeat(43)}),
            json!({"player":"7","entity":17}),
            json!({"player":"7","client_entity":17}),
            json!({"player":"7","dry":true}),
            json!({"player":"7","confirm":"server-write"}),
            json!({"player":"7","refill":true}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err(), "{value}");
        }
    }
    #[test]
    fn independent_null_zero_false_and_signed_values_never_imply_safety() {
        for (timer, inside, state) in [
            (None, None, "unavailable"),
            (None, Some(false), "partial"),
            (Some((0, 300)), None, "partial"),
            (Some((0, 300)), Some(false), "observed"),
            (Some((137, 111)), Some(true), "observed"),
            (Some((-7, 0)), Some(true), "observed"),
            (Some((i32::MIN, i32::MAX)), Some(false), "observed"),
        ] {
            let value = observation(17, timer, inside).unwrap();
            assert_eq!(value["state"], state);
            assert_eq!(value["in_shroud"], json!(inside));
            assert_eq!(
                value["timer_seconds"],
                json!(timer.map(|(current, max)| json!({"current":current,"max":max})))
            );
            assert_eq!(value["player_entity"], 17);
            assert_eq!(value["world_session"], "unavailable");
            for key in [
                "atomic_snapshot",
                "world_mutated",
                "write_authorized",
                "current_client_verified",
                "rendered_ui_verified",
                "traversal_safety_verified",
                "damage_deadline_verified",
                "refill_requested",
            ] {
                assert_eq!(value[key], false);
            }
        }
        assert!(observation(0, None, None).is_err());
    }
}
