//! Inactive shared-waypoint reader; no MCP registration, write or installation.
use enshwick::{WorldError, map::Map, world::Player};
use serde_json::{Value, json};

#[derive(Debug, PartialEq)]
pub enum ReadError {
    Native(WorldError),
    MalformedObservation,
}

/// Read one server character's shared waypoint on the server game thread.
///
/// # Errors
/// Preserve typed SDK errors; reject non-finite copied coordinates.
pub fn observe(map: Map, player: &Player) -> Result<Value, ReadError> {
    project(
        player.entity.0,
        map.waypoint(player.entity).map_err(ReadError::Native)?,
    )
}

fn project(entity: u32, position: Option<[f64; 3]>) -> Result<Value, ReadError> {
    if position.is_some_and(|point| point.iter().any(|value| !value.is_finite())) {
        return Err(ReadError::MalformedObservation);
    }
    Ok(json!({
        "source":"published_map_waypoint",
        "state":if position.is_some() {"observed"} else {"not_observed"},
        "player_entity":entity,"position_m":position,
        "entity_scope":"server character; session only, not a durable selector",
        "world_session":"unavailable",
        "missing_semantics":"off or entity without a waypoint; no default position inferred",
        "position_semantics":"shared map waypoint, not player position or arrival",
        "current_player_verified":false,"client_ui_verified":false,
        "waypoint_write_requested":false,"movement_requested":false,
        "commands_submitted":false,"rendered_verified":false,"write_authorized":false
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::abi::{Header, PlayerInfo, Status};
    use enshwick::content::EngineFacts;
    use enshwick::test::facts;
    use enshwick::{Entity, Load, Mod, Off, PlayerKey, world::World};
    use std::sync::Mutex;

    static HANDLES: Mutex<Option<(Map, World)>> = Mutex::new(None);
    const CHARACTER: Entity = Entity(17);
    struct Reader;
    impl Mod for Reader {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *HANDLES.lock().unwrap() = Some((load.map()?, load.world()?));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Reader, "id = \"test.waypoint-reader\"\nside = \"server\"");

    fn bytes(host: &enshwick::test::FakeHost, enabled: u8, at: [f64; 3]) -> Vec<u8> {
        let shown = host.offset(facts::PLAYER_WAYPOINT_ENABLED.id()).unwrap();
        let position = host.offset(facts::PLAYER_WAYPOINT_POSITION.id()).unwrap();
        let mut data = vec![0; position + 24];
        data[shown] = enabled;
        for (axis, value) in enshwick::ecs::fixed(at).iter().enumerate() {
            data[position + 8 * axis..][..8].copy_from_slice(&value.to_le_bytes());
        }
        data
    }

    fn start(
        mut host: enshwick::test::FakeHost,
        point: Option<(u8, [f64; 3])>,
    ) -> (enshwick::test::Installed, Map, Player) {
        host = host.world().player(PlayerInfo {
            header: Header::of::<PlayerInfo>(),
            slot: 0,
            entity: CHARACTER,
            key: PlayerKey::new(7),
            permissions: 0,
            name_len: 0,
            name: [0; PlayerInfo::NAME_MAX],
        });
        if let Some((enabled, at)) = point {
            let data = bytes(&host, enabled, at);
            let ty = host.hash(facts::PLAYER_WAYPOINT.id()).unwrap();
            host = host.component(CHARACTER, ty, &data);
        }
        let host = host.install();
        // SAFETY: these generated exports run only against this test's fake host.
        assert_eq!(
            unsafe { host.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        let (map, world) = HANDLES.lock().unwrap().unwrap();
        let player = world.player(CHARACTER).unwrap();
        (host, map, player)
    }

    #[test]
    fn observed_waypoint_preserves_raw_axes_and_no_control_authority() {
        let (_host, map, player) = start(
            enshwick::test::FakeHost::new().map_waypoints(),
            Some((1, [-12.125, 20.5, 30.875])),
        );
        let value = observe(map, &player).unwrap();
        assert_eq!(value["state"], "observed");
        assert_eq!(value["position_m"], json!([-12.125, 20.5, 30.875]));
        assert_eq!(value["player_entity"], 17);
        assert_eq!(value["world_session"], "unavailable");
        for key in [
            "current_player_verified",
            "client_ui_verified",
            "waypoint_write_requested",
            "movement_requested",
            "commands_submitted",
            "rendered_verified",
            "write_authorized",
        ] {
            assert_eq!(value[key], false);
        }
        assert_eq!(observe(map, &player).unwrap(), value);
    }

    #[test]
    fn unset_missing_and_legitimate_zero_are_distinct() {
        for (point, observed) in [
            (None, false),
            (Some((0, [12., 23., 34.])), false),
            (Some((1, [0.; 3])), true),
        ] {
            let (_host, map, player) =
                start(enshwick::test::FakeHost::new().map_waypoints(), point);
            let value = observe(map, &player).unwrap();
            assert_eq!(
                value["state"],
                if observed { "observed" } else { "not_observed" }
            );
            assert_eq!(
                value["position_m"],
                if observed {
                    json!([0., 0., 0.])
                } else {
                    Value::Null
                }
            );
        }
    }

    #[test]
    fn departed_character_does_not_reuse_another_characters_waypoint() {
        let (_host, map, mut player) = start(
            enshwick::test::FakeHost::new().map_waypoints(),
            Some((1, [12., 23., 34.])),
        );
        player.entity = Entity(99);
        let value = observe(map, &player).unwrap();
        assert_eq!(value["state"], "not_observed");
        assert!(value["position_m"].is_null());
    }

    #[test]
    fn wrong_thread_and_world_absence_remain_typed_errors() {
        let (host, map, player) = start(
            enshwick::test::FakeHost::new().map_waypoints(),
            Some((1, [12., 23., 34.])),
        );
        let other = player.clone();
        assert_eq!(
            std::thread::spawn(move || observe(map, &other))
                .join()
                .unwrap(),
            Err(ReadError::Native(WorldError::WrongThread))
        );
        host.stop_world();
        assert_eq!(
            observe(map, &player),
            Err(ReadError::Native(WorldError::Unavailable))
        );
    }

    #[test]
    fn unavailable_layout_is_not_an_unset_waypoint() {
        let (_host, map, player) = start(enshwick::test::FakeHost::new(), None);
        assert_eq!(
            observe(map, &player),
            Err(ReadError::Native(WorldError::Unavailable))
        );
    }

    #[test]
    fn subsequent_reads_observe_changed_and_cleared_components_without_commands() {
        let schema = enshwick::test::FakeHost::new().map_waypoints();
        let ty = schema.hash(facts::PLAYER_WAYPOINT.id()).unwrap();
        let changed = bytes(&schema, 1, [22.125, 33.5, 44.875]);
        let cleared = bytes(&schema, 0, [22.125, 33.5, 44.875]);
        let (host, map, player) = start(schema, Some((1, [12., 23., 34.])));
        assert_eq!(
            observe(map, &player).unwrap()["position_m"],
            json!([12., 23., 34.])
        );
        assert!(host.set_component(CHARACTER, ty, &changed));
        assert_eq!(
            observe(map, &player).unwrap()["position_m"],
            json!([22.125, 33.5, 44.875])
        );
        assert!(host.set_component(CHARACTER, ty, &cleared));
        assert!(observe(map, &player).unwrap()["position_m"].is_null());
    }

    #[test]
    fn malformed_copied_coordinates_refuse_without_becoming_json_null() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                project(17, Some([1., value, 3.])),
                Err(ReadError::MalformedObservation)
            );
        }
    }

    #[test]
    fn read_does_not_require_entity_command_support() {
        let (_host, map, player) = start(
            enshwick::test::FakeHost::new()
                .map_waypoints()
                .without_entity_commands(),
            Some((1, [12., 23., 34.])),
        );
        assert_eq!(
            observe(map, &player).unwrap()["position_m"],
            json!([12., 23., 34.])
        );
    }
}
