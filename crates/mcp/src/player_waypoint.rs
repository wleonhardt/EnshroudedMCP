use policy::mcp::Tool;
use serde_json::{Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "player_waypoint",
        description: "READ: one current server player's shared map waypoint, selected by exact case-insensitive name or account key. Requires active MCP world lifecycle at admission and execution with unchanged epoch; re-resolves the current SDK roster on the game thread. Unset stays null; zero coordinates stay observed; SDK failures remain errors. Session-only character identity; no world-session identifier, current client UI, player position, rendered marker, arrival, waypoint setter or movement authority.",
        schema: policy::player_waypoint::schema(),
    }
}

pub(crate) fn read(
    world: enshwick::world::World,
    map: Option<enshwick::map::Map>,
    selector: &str,
) -> Result<Value, String> {
    let player = policy::inspection::unique_player(
        world
            .players()
            .into_iter()
            .filter(|p| policy::building_access::matches_player(selector, &p.name, p.key.0)),
    )?;
    let position = map
        .ok_or("native map service unavailable")?
        .waypoint(player.entity)
        .map_err(|error| error.to_string())?;
    let mut result = policy::player_waypoint::observation(player.entity.0, position)?;
    result["world_key"] = json!(world.key());
    result["roster_match_verified"] = json!(true);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::tools::GameTools;
    use enshwick::abi::{Header, PlayerInfo, Status};
    use enshwick::content::EngineFacts;
    use enshwick::test::facts;
    use enshwick::{Entity, Load, Mod, Off, PlayerKey, WorldError};
    use policy::mcp::Tools;
    use serde_json::{Value, json};
    use std::{
        sync::{Arc, Mutex, mpsc},
        thread,
        time::{Duration, Instant},
    };

    static TOOLS: Mutex<Option<Arc<GameTools>>> = Mutex::new(None);
    static SCHEDULER: Mutex<Option<enshwick::schedule::Scheduler>> = Mutex::new(None);
    static MAP: Mutex<Option<enshwick::map::Map>> = Mutex::new(None);
    struct Reader;
    impl Mod for Reader {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *TOOLS.lock().unwrap() = Some(Arc::new(crate::Mcp::init(load)?.tools));
            *SCHEDULER.lock().unwrap() = Some(load.scheduler()?);
            *MAP.lock().unwrap() = Some(load.map()?);
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Reader, "id = \"test.player-waypoint\"\nside = \"server\"");

    fn player(entity: u32, key: u64) -> PlayerInfo {
        let mut value = PlayerInfo {
            header: Header::of::<PlayerInfo>(),
            slot: 0,
            entity: Entity(entity),
            key: PlayerKey::new(key),
            permissions: 0,
            name_len: 10,
            name: [0; PlayerInfo::NAME_MAX],
        };
        value.name[..10].copy_from_slice(b"Researcher");
        value
    }
    fn bytes(fake: &enshwick::test::FakeHost, enabled: u8, at: [f64; 3]) -> Vec<u8> {
        let shown = fake.offset(facts::PLAYER_WAYPOINT_ENABLED.id()).unwrap();
        let position = fake.offset(facts::PLAYER_WAYPOINT_POSITION.id()).unwrap();
        let mut data = vec![0; position + 24];
        data[shown] = enabled;
        for (axis, value) in enshwick::ecs::fixed(at).iter().enumerate() {
            data[position + axis * 8..][..8].copy_from_slice(&value.to_le_bytes());
        }
        data
    }
    fn start(
        mut fake: enshwick::test::FakeHost,
        players: &[PlayerInfo],
        point: Option<(u8, [f64; 3])>,
    ) -> (enshwick::test::Installed, Arc<GameTools>) {
        fake = fake.world();
        for player in players {
            fake = fake.player(*player);
        }
        if let Some((enabled, at)) = point {
            let data = bytes(&fake, enabled, at);
            let ty = fake.hash(facts::PLAYER_WAYPOINT.id()).unwrap();
            fake = fake.component(Entity(17), ty, &data);
        }
        let loader = fake.install();
        // SAFETY: these generated exports belong to this fake-only reader.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let tools = TOOLS.lock().unwrap().as_ref().unwrap().clone();
        (loader, tools)
    }
    fn request(
        loader: &enshwick::test::Installed,
        tools: &Arc<GameTools>,
        selector: &str,
    ) -> Result<Value, String> {
        let tools = Arc::clone(tools);
        let args = json!({"player":selector});
        let (tx, rx) = mpsc::sync_channel(0);
        let worker = thread::spawn(move || {
            tx.send(()).unwrap();
            tools.call("player_waypoint", args.as_object().unwrap())
        });
        rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let until = Instant::now() + Duration::from_secs(2);
        while !worker.is_finished() && Instant::now() < until {
            let _ = loader.tick(Duration::from_millis(1));
            thread::sleep(Duration::from_millis(1));
        }
        worker.join().unwrap()
    }

    #[test]
    fn advertised_read_and_entire_server_catalog_match_contracts() {
        let (_loader, tools) = start(
            enshwick::test::FakeHost::new().map_waypoints(),
            &[player(17, 7)],
            None,
        );
        assert!(tools.list().iter().any(|t| t.name == "player_waypoint"));
        let contracts: Value =
            serde_json::from_str(include_str!("../../../plans/capability-contracts.json")).unwrap();
        let expected = contracts["tools"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| t["scope"] == "server")
            .collect::<Vec<_>>();
        assert_eq!(tools.list().len(), expected.len());
        for tool in tools.list() {
            let row = expected.iter().find(|t| t["name"] == tool.name).unwrap();
            assert_eq!(row["description"], tool.description);
            assert_eq!(row["input_schema"], tool.schema);
        }
        for value in [
            json!({"player":"7","confirm":"server-write"}),
            json!({"player":"7","entity":17}),
            json!({"player":"7","dry":true}),
        ] {
            assert!(
                tools
                    .call("player_waypoint", value.as_object().unwrap())
                    .unwrap_err()
                    .contains("unknown argument")
            );
        }
    }
    #[test]
    fn listener_request_reads_on_scheduled_game_thread_with_current_roster() {
        let (loader, tools) = start(
            enshwick::test::FakeHost::new()
                .map_waypoints()
                .without_entity_commands(),
            &[player(17, 7)],
            Some((1, [-12.125, 20.5, 30.875])),
        );
        let value = request(&loader, &tools, "researcher").unwrap();
        assert_eq!(value["position_m"], json!([-12.125, 20.5, 30.875]));
        assert_eq!(value["player_entity"], 17);
        assert_eq!(value["roster_match_verified"], true);
        assert_eq!(value["movement_requested"], false);
    }
    #[test]
    fn zero_then_moved_then_cleared_waypoint_never_replays_a_cached_position() {
        let fake = enshwick::test::FakeHost::new().map_waypoints();
        let ty = fake.hash(facts::PLAYER_WAYPOINT.id()).unwrap();
        let moved = bytes(&fake, 1, [12.125, 23.5, 34.875]);
        let cleared = bytes(&fake, 0, [12.125, 23.5, 34.875]);
        let (loader, tools) = start(fake, &[player(17, 7)], Some((1, [0.; 3])));
        assert_eq!(
            request(&loader, &tools, "7").unwrap()["position_m"],
            json!([0., 0., 0.])
        );
        assert!(loader.set_component(Entity(17), ty, &moved));
        assert_eq!(
            request(&loader, &tools, "7").unwrap()["position_m"],
            json!([12.125, 23.5, 34.875])
        );
        assert!(loader.set_component(Entity(17), ty, &cleared));
        let value = request(&loader, &tools, "7").unwrap();
        assert_eq!(value["state"], "not_observed");
        assert!(value["position_m"].is_null());
    }
    #[test]
    fn absent_ambiguous_and_entity_instead_of_account_selectors_refuse() {
        let (loader, tools) = start(
            enshwick::test::FakeHost::new().map_waypoints(),
            &[player(17, 7), player(18, 8)],
            Some((1, [12., 23., 34.])),
        );
        assert!(
            request(&loader, &tools, "Researcher")
                .unwrap_err()
                .contains("ambiguous")
        );
        assert!(
            request(&loader, &tools, "17")
                .unwrap_err()
                .contains("no connected player")
        );
        assert_eq!(request(&loader, &tools, "7").unwrap()["player_entity"], 17);
    }
    #[test]
    fn missing_layout_is_an_error_instead_of_an_unset_point() {
        let (loader, tools) = start(enshwick::test::FakeHost::new(), &[player(17, 7)], None);
        assert_eq!(
            request(&loader, &tools, "7"),
            Err(WorldError::Unavailable.to_string())
        );
    }
    #[test]
    fn empty_or_stopped_world_never_replays_a_previous_player_observation() {
        let (loader, tools) = start(enshwick::test::FakeHost::new().map_waypoints(), &[], None);
        assert!(
            request(&loader, &tools, "7")
                .unwrap_err()
                .contains("no connected player")
        );
        drop(loader);
        let (loader, tools) = start(
            enshwick::test::FakeHost::new().map_waypoints(),
            &[player(17, 7)],
            Some((1, [12., 23., 34.])),
        );
        assert!(request(&loader, &tools, "7").is_ok());
        loader.stop_world();
        assert!(request(&loader, &tools, "7").is_err());
    }
    #[test]
    fn queued_reads_refuse_shutdown_or_changed_epoch_before_execution() {
        for phase in ["unchanged", "reload", "shutdown"] {
            let (loader, tools) = start(
                enshwick::test::FakeHost::new().map_waypoints(),
                &[player(17, 7)],
                Some((1, [12., 23., 34.])),
            );
            let scheduler = SCHEDULER.lock().unwrap().unwrap();
            let _ = loader.tick(Duration::from_millis(1));
            assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
            let worker = thread::spawn(move || {
                tools.call(
                    "player_waypoint",
                    json!({"player":"7"}).as_object().unwrap(),
                )
            });
            // Pinned SDK Debug only observes enqueue; it is not game execution
            // or liveness proof. This isolated scheduler has no other tasks.
            let until = Instant::now() + Duration::from_secs(2);
            while format!("{scheduler:?}") != "Scheduler { pending: true }" {
                assert!(
                    !worker.is_finished(),
                    "{phase}: request finished without queueing"
                );
                assert!(Instant::now() < until, "{phase}: queue barrier not reached");
                thread::yield_now();
            }
            match phase {
                "shutdown" => {
                    assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
                }
                "reload" => {
                    assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
                }
                _ => {}
            }
            // A retained waypoint independently proves refusal is lifecycle-based.
            assert!(
                MAP.lock()
                    .unwrap()
                    .unwrap()
                    .waypoint(Entity(17))
                    .unwrap()
                    .is_some()
            );
            let until = Instant::now() + Duration::from_secs(2);
            while !worker.is_finished() && Instant::now() < until {
                let _ = loader.tick(Duration::from_millis(1));
                thread::yield_now();
            }
            let result = worker.join().unwrap();
            if phase == "unchanged" {
                let value = result.unwrap();
                assert_eq!(value["position_m"], json!([12., 23., 34.]));
                assert_eq!(value["roster_match_verified"], true);
            } else {
                let error =
                    result.expect_err("queued changed-context read returned an observation");
                assert!(
                    error.contains(if phase == "shutdown" {
                        "active"
                    } else {
                        "session"
                    }),
                    "{phase}: {error}"
                );
            }
        }
    }
    #[test]
    fn shutdown_before_admission_refuses_even_with_retained_waypoint() {
        let (loader, tools) = start(
            enshwick::test::FakeHost::new().map_waypoints(),
            &[player(17, 7)],
            Some((1, [12., 23., 34.])),
        );
        assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
        assert!(
            MAP.lock()
                .unwrap()
                .unwrap()
                .waypoint(Entity(17))
                .unwrap()
                .is_some()
        );
        assert!(
            request(&loader, &tools, "7")
                .unwrap_err()
                .contains("active")
        );
    }
}
