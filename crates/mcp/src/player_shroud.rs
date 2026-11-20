use enshwick::world::World;
use policy::mcp::Tool;
use serde_json::{Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "player_shroud",
        description: "READ: one current SDK server-player character's shroud flag and timer, selected by exact case-insensitive name or account key. Requires active MCP world lifecycle and a nonempty SDK world key; re-resolves the roster on the game thread. Preserve independent null versus false/zero and signed SDK integer seconds; partial availability is explicit. No current client, displayed countdown, damage deadline, traversal safety, refill or movement authority.",
        schema: policy::player_shroud::schema(),
    }
}

pub(crate) fn read(world: World, selector: &str) -> Result<Value, String> {
    let world_key = world.key();
    policy::player_shroud::require_world(world_key.as_deref())?;
    let player = policy::inspection::unique_player(
        world
            .players()
            .into_iter()
            .filter(|p| policy::building_access::matches_player(selector, &p.name, p.key.0)),
    )?;
    let timer = world.shroud_time(player.entity).map(|m| (m.current, m.max));
    let inside = world.in_shroud(player.entity);
    let mut result = policy::player_shroud::observation(player.entity.0, timer, inside)?;
    result["world_key"] = json!(world_key);
    result["roster_match_verified"] = json!(true);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::tools::GameTools;
    use enshwick::{
        Entity, Load, Mod, Off, PlayerKey,
        abi::{Header, PlayerInfo, Status},
    };
    use policy::mcp::Tools;
    use serde_json::{Value, json};
    use std::{
        sync::{Arc, Mutex, mpsc},
        thread,
        time::{Duration, Instant},
    };
    static TOOLS: Mutex<Option<Arc<GameTools>>> = Mutex::new(None);
    static WORLD: Mutex<Option<enshwick::world::World>> = Mutex::new(None);
    static SCHEDULER: Mutex<Option<enshwick::schedule::Scheduler>> = Mutex::new(None);
    const CHARACTER: Entity = Entity(17);
    struct Reader;
    impl Mod for Reader {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *TOOLS.lock().unwrap() = Some(Arc::new(crate::Mcp::init(load)?.tools));
            *WORLD.lock().unwrap() = Some(load.world()?);
            *SCHEDULER.lock().unwrap() = Some(load.scheduler()?);
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Reader, "id = \"test.player-shroud\"\nside = \"server\"");
    fn timer(current: i32, max: i32) -> Vec<u8> {
        let mut bytes = 60u32.to_le_bytes().to_vec();
        bytes.extend(8u16.to_le_bytes());
        bytes.extend([1, 2]);
        bytes.extend(current.to_le_bytes());
        bytes.extend(max.to_le_bytes());
        bytes
    }
    fn host(timer_layout: bool, actor_layout: bool) -> enshwick::test::FakeHost {
        let mut host = enshwick::test::FakeHost::new()
            .world()
            .without_entity_commands()
            .constant("ecs.level_data_offset", 4)
            .constant("ecs.level_flags", 6)
            .constant("ecs.attribute_count", 7);
        for (index, name) in ["health", "stamina", "mana", "level", "experience"]
            .iter()
            .enumerate()
        {
            host = host
                .type_hash(&format!("attribute_set.{name}"), index as u32 + 1)
                .constant(&format!("attribute.{name}"), (index as u64 + 1) * 10);
        }
        for name in ["health", "stamina", "mana"] {
            host = host.constant(&format!("attribute.{name}_max_slot"), 1);
        }
        host = host.constant("attribute.experience_needed_slot", 1);
        if timer_layout {
            host = host
                .type_hash("attribute_set.shroud", 6)
                .constant("attribute.shroud", 60)
                .constant("attribute.shroud_max_slot", 1);
        }
        if actor_layout {
            host = host
                .type_hash("base_actor", 7)
                .type_size("base_actor", 8)
                .constant("base_actor.state", 0)
                .constant("actor.state_in_shroud", 4);
        }
        host
    }
    fn player(entity: u32, key: u64) -> PlayerInfo {
        let mut row = PlayerInfo {
            header: Header::of::<PlayerInfo>(),
            slot: 0,
            entity: Entity(entity),
            key: PlayerKey::new(key),
            permissions: 0,
            name_len: 10,
            name: [0; PlayerInfo::NAME_MAX],
        };
        row.name[..10].copy_from_slice(b"Researcher");
        row
    }
    fn start(
        mut host: enshwick::test::FakeHost,
        players: &[PlayerInfo],
    ) -> (enshwick::test::Installed, Arc<GameTools>) {
        for row in players {
            host = host.player(*row);
        }
        let loader = host.world_key("fixture-shroud-world").install();
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
        let copied = Arc::clone(tools);
        let args = json!({"player":selector});
        let (tx, rx) = mpsc::sync_channel(0);
        let worker = thread::spawn(move || {
            tx.send(()).unwrap();
            copied.call("player_shroud", args.as_object().unwrap())
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
    fn registered_read_uses_scheduled_roster_and_typed_shroud_getters() {
        let (loader, tools) = start(
            host(true, true)
                .component(CHARACTER, 6, &timer(60, 300))
                .component(CHARACTER, 7, &4u64.to_le_bytes()),
            &[player(17, 7)],
        );
        assert_eq!(
            WORLD
                .lock()
                .unwrap()
                .unwrap()
                .shroud_time(CHARACTER)
                .unwrap()
                .current,
            60
        );
        assert_eq!(
            WORLD.lock().unwrap().unwrap().in_shroud(CHARACTER),
            Some(true)
        );
        assert!(tools.list().iter().any(|t| t.name == "player_shroud"));
        let value = request(&loader, &tools, "researcher").unwrap();
        assert_eq!(value["player_entity"], 17);
        assert_eq!(value["timer_seconds"], json!({"current":60,"max":300}));
        assert_eq!(value["in_shroud"], true);
        assert_eq!(value["roster_match_verified"], true);
        assert_eq!(value["world_key"], "fixture-shroud-world");
        assert_eq!(value["world_mutated"], false);
    }

    #[test]
    fn partial_unavailable_and_signed_values_keep_no_safety_authority() {
        for (pool, inside) in [
            (None, None),
            (Some((0, 300)), None),
            (None, Some(false)),
            (Some((-7, 0)), Some(true)),
            (Some((i32::MIN, i32::MAX)), Some(false)),
        ] {
            let mut fake = host(true, true);
            if let Some((current, max)) = pool {
                fake = fake.component(CHARACTER, 6, &timer(current, max));
            }
            if let Some(inside) = inside {
                fake = fake.component(CHARACTER, 7, &(if inside { 4u64 } else { 0 }).to_le_bytes());
            }
            let (loader, tools) = start(fake, &[player(17, 7)]);
            let value = request(&loader, &tools, "7").unwrap();
            assert_eq!(
                value["timer_seconds"],
                json!(pool.map(|(current, max)| json!({"current":current,"max":max})))
            );
            assert_eq!(value["in_shroud"], json!(inside));
            assert_eq!(
                value["state"],
                if pool.is_some() && inside.is_some() {
                    "observed"
                } else if pool.is_none() && inside.is_none() {
                    "unavailable"
                } else {
                    "partial"
                }
            );
            for key in [
                "world_mutated",
                "write_authorized",
                "current_client_verified",
                "rendered_ui_verified",
                "traversal_safety_verified",
                "damage_deadline_verified",
                "refill_requested",
                "atomic_snapshot",
            ] {
                assert_eq!(value[key], false);
            }
        }
    }
    #[test]
    fn each_request_reads_current_values_and_stopped_world_never_replays() {
        let (loader, tools) = start(
            host(true, true)
                .component(CHARACTER, 6, &timer(60, 300))
                .component(CHARACTER, 7, &4u64.to_le_bytes()),
            &[player(17, 7)],
        );
        assert_eq!(
            request(&loader, &tools, "7").unwrap()["timer_seconds"]["current"],
            60
        );
        assert!(loader.set_component(CHARACTER, 6, &timer(0, 300)));
        assert!(loader.set_component(CHARACTER, 7, &0u64.to_le_bytes()));
        let value = request(&loader, &tools, "7").unwrap();
        assert_eq!(value["timer_seconds"]["current"], 0);
        assert_eq!(value["in_shroud"], false);
        let old_context = value["context_session"].clone();
        loader.stop_world();
        // The fake read-stop helper does not publish lifecycle events itself.
        assert!(loader.shut_down());
        assert!(request(&loader, &tools, "7").is_err());
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let fresh = request(&loader, &tools, "7").unwrap();
        assert_ne!(fresh["context_session"], old_context);
        assert_eq!(fresh["timer_seconds"]["current"], 0);
    }
    #[test]
    fn missing_ambiguous_and_entity_id_selectors_refuse_current_roster_selection() {
        for (players, selector, error) in [
            (vec![], "7", "no connected player"),
            (
                vec![player(17, 7), player(18, 8)],
                "researcher",
                "ambiguous",
            ),
            (vec![player(17, 7)], "17", "no connected player"),
        ] {
            let (loader, tools) = start(host(true, true), &players);
            assert!(
                request(&loader, &tools, selector)
                    .unwrap_err()
                    .contains(error)
            );
        }
        let (loader, tools) = start(host(true, true), &[player(17, 7), player(18, 8)]);
        assert_eq!(request(&loader, &tools, "8").unwrap()["player_entity"], 18);
    }
    #[test]
    fn invalid_arguments_and_zero_native_character_never_produce_an_observation() {
        let (loader, tools) = start(host(true, true), &[player(0, 7)]);
        for args in [
            json!({}),
            json!({"player":"7","entity":17}),
            json!({"player":"7","client_entity":17}),
            json!({"player":"7","dry":true}),
            json!({"player":"7","confirm":"server-write"}),
            json!({"player":" padded"}),
        ] {
            assert!(
                tools
                    .call("player_shroud", args.as_object().unwrap())
                    .is_err()
            );
        }
        assert!(
            request(&loader, &tools, "7")
                .unwrap_err()
                .contains("zero server entity")
        );
    }

    #[test]
    fn queued_reads_refuse_shutdown_or_changed_epoch_before_execution() {
        for phase in ["unchanged", "shutdown", "reload"] {
            let (loader, tools) = start(
                host(true, true)
                    .component(CHARACTER, 6, &timer(60, 300))
                    .component(CHARACTER, 7, &4u64.to_le_bytes()),
                &[player(17, 7)],
            );
            let scheduler = SCHEDULER.lock().unwrap().unwrap();
            let _ = loader.tick(Duration::from_millis(1));
            assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
            let worker = thread::spawn(move || {
                tools.call("player_shroud", json!({"player":"7"}).as_object().unwrap())
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
            // The SDK fixture still supplies data. A changed-context refusal
            // must come from the lifecycle guard, not a missing component.
            assert_eq!(
                WORLD
                    .lock()
                    .unwrap()
                    .unwrap()
                    .shroud_time(CHARACTER)
                    .unwrap()
                    .current,
                60
            );
            let until = Instant::now() + Duration::from_secs(2);
            while !worker.is_finished() && Instant::now() < until {
                let _ = loader.tick(Duration::from_millis(1));
                thread::yield_now();
            }
            let result = worker.join().unwrap();
            if phase == "unchanged" {
                let value = result.unwrap();
                assert_eq!(value["timer_seconds"]["current"], 60);
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
}
