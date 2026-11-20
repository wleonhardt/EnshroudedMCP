//! Strict read-only player selection and shared-waypoint projection.
use serde_json::{Value, json};

pub use crate::interaction_target::{check, schema};

pub fn observation(entity: u32, position: Option<[f64; 3]>) -> Result<Value, String> {
    if entity == 0 || position.is_some_and(|at| at.iter().any(|value| !value.is_finite())) {
        return Err("native waypoint observation is malformed".into());
    }
    Ok(
        json!({"source":"published_map_waypoint","player_entity":entity,
        "state":if position.is_some(){"observed"}else{"not_observed"},"position_m":position,
        "world_session":"unavailable","entity_scope":"server character; session only, not a durable selector",
        "missing_semantics":"off or entity without a waypoint; no default position inferred",
        "position_semantics":"shared waypoint; not player position, rendered marker or arrival",
        "current_client_verified":false,"rendered_verified":false,
        "waypoint_write_requested":false,"movement_requested":false,"world_mutated":false,"write_authorized":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_selection_is_bounded_and_has_no_mutation_fields() {
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
            json!({"player":"7","dry":true}),
            json!({"player":"7","confirm":"server-write"}),
            json!({"player":"7","position":[1,2,3]}),
        ] {
            assert!(check(value.as_object().unwrap()).is_err(), "{value}");
        }
    }

    #[test]
    fn absent_zero_raw_positions_and_malformed_observations_remain_distinct() {
        let none = observation(17, None).unwrap();
        assert_eq!(none["state"], "not_observed");
        assert!(none["position_m"].is_null());
        for at in [[0.; 3], [-12.125, 20.5, 30.875]] {
            let value = observation(17, Some(at)).unwrap();
            assert_eq!(value["state"], "observed");
            assert_eq!(value["position_m"], json!(at));
            assert_eq!(value["player_entity"], 17);
            assert_eq!(value["world_session"], "unavailable");
            for key in [
                "current_client_verified",
                "rendered_verified",
                "waypoint_write_requested",
                "movement_requested",
                "write_authorized",
            ] {
                assert_eq!(value[key], false);
            }
        }
        assert!(observation(0, None).is_err());
        for at in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(observation(17, Some([1., at, 3.])).is_err());
        }
    }
}
