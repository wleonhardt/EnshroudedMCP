//! Inactive read-only respawn-target consumer; no product tool or native install.
use enshwick::{
    WorldError,
    world::{Player, PlayerSpawnType, RespawnPoint, World},
};
use serde_json::{Value, json};

#[derive(Debug, PartialEq)]
pub enum ReadError {
    Native(WorldError),
    MalformedObservation,
}

/// Reads only the future respawn target, on the server game thread.
///
/// # Errors
/// Preserves native thread/availability errors and rejects non-JSON observations.
pub fn observe(world: World, player: &Player) -> Result<Value, ReadError> {
    project(world.respawn_point(player).map_err(ReadError::Native)?)
}

fn project(point: Option<RespawnPoint>) -> Result<Value, ReadError> {
    let (state, value) = match point {
        None => ("not_observed", None),
        Some(point) => {
            if point.entity.0 == 0
                || !point
                    .transform
                    .position
                    .iter()
                    .all(|value| value.is_finite())
                || !point
                    .transform
                    .orientation
                    .iter()
                    .all(|value| value.is_finite())
            {
                return Err(ReadError::MalformedObservation);
            }
            let kind = match point.kind {
                PlayerSpawnType::StartLocation => "start_location",
                PlayerSpawnType::SavePoint => "save_point",
                PlayerSpawnType::Base => "base",
                PlayerSpawnType::NoRespawn => "no_respawn",
                _ => "other",
            };
            (
                "observed",
                Some(json!({"entity":point.entity.0,"kind":kind,
                "position_m":point.transform.position,"orientation_xyzw":point.transform.orientation})),
            )
        }
    };
    Ok(
        json!({"source":"published_world_respawn_point","state":state,"point":value,
        "entity_scope":"server session only; not a durable selector","world_session":"unavailable",
        "missing_semantics":"unresolved point, fallback or departed player; no default position inferred",
        "movement_requested":false,"respawn_change_requested":false,"health_change_requested":false,
        "arrival_verified":false,"write_authorized":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{
        Entity, Load, Mod, Off, PlayerKey,
        abi::{Header, PlayerInfo, RespawnPointInfo, Status},
    };
    use std::sync::Mutex;

    static WORLD: Mutex<Option<World>> = Mutex::new(None);
    const CHARACTER: Entity = Entity(17);
    const POINT: Entity = Entity(35);
    struct Reader;
    impl Mod for Reader {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *WORLD.lock().unwrap() = Some(load.world()?);
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Reader,
        "id = \"test.respawn-observation\"\nside = \"server\""
    );

    fn start(
        host: enshwick::test::FakeHost,
        point: Option<(u32, [f64; 3])>,
    ) -> (enshwick::test::Installed, World, Player) {
        let info = PlayerInfo {
            header: Header::of::<PlayerInfo>(),
            slot: 0,
            entity: CHARACTER,
            key: PlayerKey::new(7),
            permissions: 0,
            name_len: 0,
            name: [0; PlayerInfo::NAME_MAX],
        };
        let mut host = host.world().player(info);
        if let Some((kind, position)) = point {
            host = host
                .spawn_point(POINT, kind, position)
                .respawns_at(CHARACTER, POINT);
        }
        let host = host.install();
        // SAFETY: this module's own generated exports run only against its fake host.
        assert_eq!(
            unsafe { host.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        let world = WORLD.lock().unwrap().unwrap();
        let player = world.player(CHARACTER).unwrap();
        (host, world, player)
    }

    #[test]
    fn observed_target_copies_raw_transform_without_movement_or_write_authority() {
        assert_eq!(enshwick::abi::ABI_MAJOR, 12);
        assert_eq!(enshwick::abi::ABI_MINOR, 13);
        let (_host, world, player) = start(
            enshwick::test::FakeHost::new(),
            Some((RespawnPointInfo::BASE, [-12.25, 20.5, 30.75])),
        );
        let value = observe(world, &player).unwrap();
        assert_eq!(value["state"], "observed");
        assert_eq!(
            value["point"],
            json!({"entity":35,"kind":"base","position_m":[-12.25,20.5,30.75],"orientation_xyzw":[0.0,0.0,0.0,1.0]})
        );
        for field in [
            "movement_requested",
            "respawn_change_requested",
            "health_change_requested",
            "arrival_verified",
            "write_authorized",
        ] {
            assert_eq!(value[field], false);
        }
        assert_eq!(observe(world, &player).unwrap(), value);
    }

    #[test]
    fn unresolved_point_never_invents_a_default_location() {
        let (_host, world, player) = start(enshwick::test::FakeHost::new(), None);
        let value = observe(world, &player).unwrap();
        assert_eq!(value["state"], "not_observed");
        assert!(value["point"].is_null());
        assert!(
            value["missing_semantics"]
                .as_str()
                .unwrap()
                .contains("no default position")
        );
    }

    #[test]
    fn departed_player_remains_unresolved_instead_of_using_a_stale_target() {
        let (_host, world, mut player) = start(
            enshwick::test::FakeHost::new(),
            Some((RespawnPointInfo::BASE, [10.0, 20.0, 30.0])),
        );
        player.entity = Entity(99);
        let value = observe(world, &player).unwrap();
        assert_eq!(value["state"], "not_observed");
        assert!(value["point"].is_null());
    }

    #[test]
    fn unknown_and_nonrespawn_kinds_remain_distinct_from_missing_and_zero_origin() {
        for (kind, expected) in [
            (RespawnPointInfo::START_LOCATION, "start_location"),
            (RespawnPointInfo::SAVE_POINT, "save_point"),
            (RespawnPointInfo::NO_RESPAWN, "no_respawn"),
            (777, "other"),
        ] {
            let (_host, world, player) =
                start(enshwick::test::FakeHost::new(), Some((kind, [0.0; 3])));
            let value = observe(world, &player).unwrap();
            assert_eq!(value["state"], "observed");
            assert_eq!(value["point"]["kind"], expected);
            assert_eq!(value["point"]["position_m"], json!([0.0, 0.0, 0.0]));
        }
    }

    #[test]
    fn native_thread_and_world_errors_are_never_converted_to_missing_points() {
        let (host, world, player) = start(
            enshwick::test::FakeHost::new(),
            Some((RespawnPointInfo::BASE, [10.0, 20.0, 30.0])),
        );
        let elsewhere = player.clone();
        assert_eq!(
            std::thread::spawn(move || observe(world, &elsewhere))
                .join()
                .unwrap(),
            Err(ReadError::Native(WorldError::WrongThread))
        );
        host.stop_world();
        assert_eq!(
            observe(world, &player),
            Err(ReadError::Native(WorldError::Unavailable))
        );
    }

    #[test]
    fn read_remains_available_without_entity_command_support() {
        let (_host, world, player) = start(
            enshwick::test::FakeHost::new().without_entity_commands(),
            Some((RespawnPointInfo::BASE, [10.0, 20.0, 30.0])),
        );
        assert_eq!(observe(world, &player).unwrap()["point"]["entity"], 35);
    }

    #[test]
    fn malformed_copied_values_refuse_instead_of_serializing_null_or_default_points() {
        let (_host, world, player) = start(
            enshwick::test::FakeHost::new(),
            Some((RespawnPointInfo::BASE, [10.0, 20.0, 30.0])),
        );
        let original = world.respawn_point(&player).unwrap().unwrap();
        let mut point = original;
        point.entity = Entity(0);
        assert_eq!(project(Some(point)), Err(ReadError::MalformedObservation));
        point = original;
        point.transform.position[1] = f64::NAN;
        assert_eq!(project(Some(point)), Err(ReadError::MalformedObservation));
        point = original;
        point.transform.orientation[2] = f32::INFINITY;
        assert_eq!(project(Some(point)), Err(ReadError::MalformedObservation));
    }
}
