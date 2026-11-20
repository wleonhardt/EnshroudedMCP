use enshwick::{buffs::Buffs, world::World};
use serde_json::{Value, json};

pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "player_buffs_read",
        description: "READ: active buffs for one current SDK server-player character, selected by exact case-insensitive name or account key. Requires active MCP lifecycle at admission and execution with unchanged epoch; re-resolves the roster on the game thread. At most 64 output rows, default 32; SDK first retrieves the full entity list. Preserve SDK slot, optional level and exact game-time duration nanoseconds as strings. Read errors remain errors, not empty/healthy. No current-client mapping, complete effects, actor safety, rendered UI or mutation authority.",
        schema: policy::player_buffs::schema(),
    }
}

pub(crate) fn read(
    world: World,
    buffs: Buffs,
    selector: &str,
    limit: usize,
) -> Result<Value, String> {
    let player = policy::inspection::unique_player(
        world
            .players()
            .into_iter()
            .filter(|p| policy::building_access::matches_player(selector, &p.name, p.key.0)),
    )?;
    let (entity, limit) = policy::buff_read::check(
        json!({"entity":player.entity.0,"limit":limit})
            .as_object()
            .unwrap(),
    )?;
    let mut result = crate::buff_read::read(buffs, entity, limit)?;
    result["player"] = json!({"key":player.key.0.to_string(),"entity":entity});
    result["world_key"] = json!(world.key());
    result["roster_match_verified"] = json!(true);
    result["current_client_verified"] = json!(false);
    result["world_session"] = json!("unavailable");
    result["atomic_snapshot"] = json!(false);
    result["world_mutated"] = json!(false);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::tools::GameTools;
    use enshwick::{
        Entity, Load, Mod, Off, PlayerKey,
        abi::{BuffTypeInfo, Header, PlayerInfo, Status},
        buffs::{Apply, BuffTypeId, Buffs},
        schedule::Scheduler,
        world::World,
    };
    use policy::mcp::Tools;
    use serde_json::{Value, json};
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::{Duration, Instant},
    };
    #[derive(Clone)]
    struct Context {
        tools: Arc<GameTools>,
        world: World,
        buffs: Buffs,
        scheduler: Scheduler,
    }
    static CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
    struct Reader;
    impl Mod for Reader {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *CONTEXT.lock().unwrap() = Some(Context {
                tools: Arc::new(crate::Mcp::init(load)?.tools),
                world: load.world()?,
                buffs: load.buffs(),
                scheduler: load.scheduler()?,
            });
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Reader, "id = \"test.player-buffs\"\nside = \"server\"");
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
    fn types() -> Vec<BuffTypeInfo> {
        (1..=3)
            .map(|id| BuffTypeInfo {
                header: Header::of::<BuffTypeInfo>(),
                id: BuffTypeId(id),
                slot: BuffTypeInfo::BUFF,
                flags: 0,
                name: 0,
                duration_ns: 100_000_000_000,
            })
            .collect()
    }
    fn start(players: &[PlayerInfo], populated: bool) -> (enshwick::test::Installed, Context) {
        let mut host = enshwick::test::FakeHost::new().world();
        for row in players {
            host = host.player(*row);
        }
        let loader = host.install();
        // SAFETY: generated exports belong to this isolated fake-only reader.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let context = CONTEXT.lock().unwrap().as_ref().unwrap().clone();
        if populated {
            loader.set_buffs(&types(), &[Entity(17)]);
            for id in 1..=3 {
                context
                    .buffs
                    .apply(Entity(17), BuffTypeId(id), Apply::new().level(1))
                    .unwrap();
            }
        }
        (loader, context)
    }
    fn request(
        loader: &enshwick::test::Installed,
        context: &Context,
        args: Value,
    ) -> Result<Value, String> {
        let tools = context.tools.clone();
        let worker =
            thread::spawn(move || tools.call("player_buffs_read", args.as_object().unwrap()));
        let until = Instant::now() + Duration::from_secs(2);
        while !worker.is_finished() && Instant::now() < until {
            let _ = loader.tick(Duration::from_millis(1));
            thread::yield_now();
        }
        worker.join().unwrap()
    }
    #[test]
    fn advertised_current_player_read_refuses_entity_and_write_arguments() {
        let (_loader, c) = start(&[player(17, 7)], true);
        assert!(
            c.tools.list().iter().any(|t| t.name == "player_buffs_read"),
            "current-player buff tool is not registered"
        );
        for args in [
            json!({"player":"7","entity":17}),
            json!({"player":"7","apply":1}),
            json!({"player":"7","dry":true}),
            json!({"player":"7","confirm":"server-write"}),
        ] {
            assert!(
                c.tools
                    .call("player_buffs_read", args.as_object().unwrap())
                    .unwrap_err()
                    .contains("unknown argument")
            );
        }
    }
    #[test]
    fn current_roster_selection_and_bounded_projection_preserve_sdk_data() {
        let (loader, c) = start(&[player(17, 7)], true);
        assert_eq!(c.buffs.of(Entity(17)).unwrap().len(), 3);
        let value = request(&loader, &c, json!({"player":"researcher","limit":1})).unwrap();
        assert_eq!(value["entity"], 17);
        assert_eq!(value["player"]["key"], "7");
        assert_eq!(value["total_returned_by_sdk"], 3);
        assert_eq!(value["buffs"].as_array().unwrap().len(), 1);
        assert_eq!(value["truncated"], true);
        assert_eq!(value["buffs"][0]["level"], 1);
        assert_eq!(value["buffs"][0]["lifetime_game_ns"], "100000000000");
        assert_eq!(value["roster_match_verified"], true);
        assert_eq!(value["current_client_verified"], false);
        assert_eq!(value["world_mutated"], false);
        assert_eq!(value["actor_safety_verified"], false);
        assert_eq!(value["atomic_snapshot"], false);
        assert_eq!(value["world_session"], "unavailable");
        assert!(
            value["context_session"]
                .as_str()
                .is_some_and(|s| !s.is_empty())
        );
        let value = request(&loader, &c, json!({"player":"7"})).unwrap();
        assert_eq!(value["buffs"].as_array().unwrap().len(), 3);
        assert_eq!(value["truncated"], false);
    }
    #[test]
    fn ambiguous_missing_entity_id_and_zero_native_entity_refuse() {
        let (loader, c) = start(&[player(17, 7), player(18, 8)], true);
        assert!(
            request(&loader, &c, json!({"player":"Researcher"}))
                .unwrap_err()
                .contains("ambiguous")
        );
        assert!(
            request(&loader, &c, json!({"player":"17"}))
                .unwrap_err()
                .contains("no connected player")
        );
        assert!(
            request(&loader, &c, json!({"player":"missing"}))
                .unwrap_err()
                .contains("no connected player")
        );
        drop(loader);
        let (loader, c) = start(&[player(0, 7)], false);
        assert_eq!(c.world.players()[0].entity, Entity(0));
        let error = request(&loader, &c, json!({"player":"7"})).unwrap_err();
        assert!(
            error.contains("entity") && !error.contains("active buffs read failed"),
            "{error}"
        );
    }
    #[test]
    fn unavailable_empty_and_changed_buffs_never_replay_a_cached_list() {
        let (loader, c) = start(&[player(17, 7)], false);
        assert!(
            request(&loader, &c, json!({"player":"7"}))
                .unwrap_err()
                .starts_with("active buffs read failed")
        );
        loader.set_buffs(&types(), &[Entity(17)]);
        let empty = request(&loader, &c, json!({"player":"7"})).unwrap();
        assert_eq!(empty["buffs"], json!([]));
        c.buffs
            .apply(Entity(17), BuffTypeId(1), Apply::new().level(7))
            .unwrap();
        assert_eq!(
            request(&loader, &c, json!({"player":"7"})).unwrap()["buffs"][0]["level"],
            7
        );
        loader.set_buffs(&types(), &[Entity(17)]);
        assert_eq!(
            request(&loader, &c, json!({"player":"7"})).unwrap()["buffs"],
            json!([])
        );
    }
    #[test]
    fn queued_player_buff_reads_refuse_changed_lifecycle_with_retained_roster_and_buffs() {
        for phase in ["unchanged", "reload", "shutdown"] {
            let (loader, c) = start(&[player(17, 7)], true);
            let _ = loader.tick(Duration::from_millis(1));
            let scheduler = c.scheduler;
            assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
            let tools = c.tools.clone();
            let worker = thread::spawn(move || {
                tools.call(
                    "player_buffs_read",
                    json!({"player":"7"}).as_object().unwrap(),
                )
            });
            // Pinned SDK Debug is only an enqueue barrier for this isolated queue.
            let until = Instant::now() + Duration::from_secs(2);
            while format!("{scheduler:?}") != "Scheduler { pending: true }" {
                assert!(!worker.is_finished(), "{phase}: read did not queue");
                assert!(Instant::now() < until, "queue barrier");
                thread::yield_now();
            }
            match phase {
                "reload" => assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0),
                "shutdown" => assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0),
                _ => {}
            }
            assert_eq!(c.world.players()[0].entity, Entity(17));
            assert_eq!(c.buffs.of(Entity(17)).unwrap().len(), 3);
            let until = Instant::now() + Duration::from_secs(2);
            while !worker.is_finished() && Instant::now() < until {
                let _ = loader.tick(Duration::from_millis(1));
                thread::yield_now();
            }
            let result = worker.join().unwrap();
            if phase == "unchanged" {
                assert_eq!(result.unwrap()["entity"], 17);
            } else {
                let error = result
                    .expect_err("queued changed-context player buff read returned an observation");
                assert!(
                    error.contains(if phase == "reload" {
                        "session"
                    } else {
                        "active"
                    }),
                    "{phase}: {error}"
                );
            }
        }
    }
    #[test]
    fn shutdown_before_player_buff_admission_refuses_retained_roster_and_data() {
        let (loader, c) = start(&[player(17, 7)], true);
        assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
        assert_eq!(c.buffs.of(Entity(17)).unwrap().len(), 3);
        assert!(
            c.tools
                .call(
                    "player_buffs_read",
                    json!({"player":"7"}).as_object().unwrap()
                )
                .unwrap_err()
                .contains("active")
        );
    }
}
