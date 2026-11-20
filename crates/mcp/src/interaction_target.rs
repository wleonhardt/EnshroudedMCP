//! One server prompt target; not a terrain raycast or interaction executor.
use enshwick::{Entity, interactions::Targets};
use serde_json::{Value, json};

pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "player_interaction_target",
        description: "Read one uniquely selected connected server player's native interaction-prompt target. Requires active MCP world lifecycle at admission and execution with unchanged epoch. Session-only server entity, optional position/template and native UsedItem ID; unavailable differs from no target. Template uses file-order hex. Item ID is an observation, not a resolved effective definition. Not a terrain hit, camera raycast, collision test, client entity mapping or interaction executor. No writes.",
        schema: policy::interaction_target::schema(),
    }
}

pub(crate) fn observe(
    result: Result<Option<Entity>, String>,
    mut metadata: impl FnMut(Entity) -> (Option<[f64; 3]>, Option<[u8; 16]>, Option<u32>),
) -> Value {
    match result {
        Err(error) => json!({"state":"unknown","target":null,"error":error}),
        Ok(None) => json!({"state":"none","target":null}),
        Ok(Some(entity)) => {
            let (position, template, item) = metadata(entity);
            json!({"state":"observed","target":{"entity":entity.0,"position":position,"item_id":item,
                "template_guid_encoding":"lowercase hex; 16 GUID bytes in file order",
                "template_guid":template.map(|bytes|bytes.iter().map(|byte|format!("{byte:02x}")).collect::<String>())}})
        }
    }
}

pub(crate) fn read(game: crate::tools::Game, selector: &str) -> Result<Value, String> {
    let player = policy::inspection::unique_player(
        game.world
            .players()
            .into_iter()
            .filter(|p| policy::building_access::matches_player(selector, &p.name, p.key.0)),
    )?;
    let result = game
        .interaction_targets
        .ok_or_else(|| "native interaction target reader unavailable".to_owned())
        .and_then(|targets: Targets| targets.target(player.entity).map_err(|e| e.to_string()));
    Ok(
        json!({"world_key":game.world.key(),"player":{"key":player.key.0.to_string(),"entity":player.entity.0},
        "observation":observe(result, |entity|(game.world.position(entity),game.world.template(entity).map(|guid|guid.0),game.world.used_item(entity).map(|item|item.0))),
        "world_mutated":false,"scope":"server prompt target only; session entity ids; no terrain hit, rendered contact or interaction execution"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{Load, Mod, Off};
    use std::{sync::Mutex, time::Duration};
    static OBSERVATIONS: Mutex<Vec<Value>> = Mutex::new(Vec::new());
    struct Probe;
    impl Mod for Probe {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            let targets = load.interaction_targets()?;
            let world = load.world()?;
            load.on_tick(Duration::from_millis(100), move |_| {
                OBSERVATIONS.lock().unwrap().push(observe(
                    targets.target(Entity(40)).map_err(|e| e.to_string()),
                    |entity| {
                        (
                            Some([1., 2., 3.]),
                            world.template(entity).map(|guid| guid.0),
                            world.used_item(entity).map(|item| item.0),
                        )
                    },
                ));
            })?;
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.prompt-target"
side = "server""#
    );
    #[test]
    fn published_reader_observes_prompt_slot_instead_of_combat_target() {
        use enshwick::{content::EngineFacts, test::facts};
        let fake = enshwick::test::FakeHost::new()
            .world()
            .interaction_targets();
        let used_item = fake.hash(facts::USED_ITEM.id()).unwrap();
        let ty = fake.hash(facts::SELECTED_TARGETS.id()).unwrap();
        let size = fake.size(facts::SELECTED_TARGETS.id()).unwrap();
        let stride = fake.size(facts::TARGET_INFO.id()).unwrap();
        let offset = fake.offset(facts::TARGET_INFO_ENTITY.id()).unwrap();
        let mut bytes = vec![0; size];
        bytes[offset..offset + 4].copy_from_slice(&77u32.to_le_bytes());
        let loader = fake
            .component(Entity(40), ty, &bytes)
            .component(Entity(512), used_item, &41u32.to_le_bytes())
            .template(
                Entity(512),
                enshwick::content::Guid(std::array::from_fn(|i| i as u8)),
            )
            .install();
        // SAFETY: these exports belong to this test module's Probe.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let _ = loader.tick(Duration::from_millis(100));
        let at = stride * 2 + offset;
        bytes[at..at + 4].copy_from_slice(&512u32.to_le_bytes());
        assert!(loader.set_component(Entity(40), ty, &bytes));
        let _ = loader.tick(Duration::from_millis(100));
        assert!(loader.set_component(Entity(512), used_item, &0u32.to_le_bytes()));
        let _ = loader.tick(Duration::from_millis(100));
        bytes[at..at + 4].copy_from_slice(&513u32.to_le_bytes());
        assert!(loader.set_component(Entity(40), ty, &bytes));
        let _ = loader.tick(Duration::from_millis(100));
        let seen = OBSERVATIONS.lock().unwrap();
        assert_eq!(seen[0]["state"], "none");
        assert_eq!(seen[1]["target"]["entity"], 512);
        assert_eq!(seen[1]["target"]["item_id"], 41);
        assert_eq!(seen[2]["target"]["item_id"], 0);
        assert!(seen[3]["target"].get("item_id").is_some_and(Value::is_null));
        assert_eq!(seen[1]["target"]["position"], json!([1., 2., 3.]));
        assert_eq!(
            seen[1]["target"]["template_guid"],
            "000102030405060708090a0b0c0d0e0f"
        );
    }
    #[test]
    fn unknown_and_no_target_never_read_metadata_or_become_each_other() {
        assert_eq!(
            observe(Err("missing reader".into()), |_| panic!())["state"],
            "unknown"
        );
        assert_eq!(observe(Ok(None), |_| panic!())["state"], "none");
    }
    #[test]
    fn only_the_observed_target_is_inspected_and_missing_metadata_stays_null() {
        let mut inspected = Vec::new();
        let result = observe(Ok(Some(Entity(7))), |entity| {
            inspected.push(entity.0);
            (None, None, None)
        });
        assert_eq!(inspected, vec![7]);
        assert_eq!(result["state"], "observed");
        assert_eq!(result["target"]["entity"], 7);
        assert!(result["target"]["position"].is_null());
        assert!(result["target"]["template_guid"].is_null());
        assert!(result["target"].get("item_id").is_some_and(Value::is_null));
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use crate::tools::GameTools;
    use enshwick::{
        Entity, Load, Mod, Off, PlayerKey,
        abi::{Header, PlayerInfo, Status},
        content::EngineFacts,
        interactions::Targets,
        schedule::Scheduler,
        test::facts,
        world::World,
    };
    use policy::mcp::Tools;
    use serde_json::json;
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::{Duration, Instant},
    };
    #[derive(Clone)]
    struct Context {
        tools: Arc<GameTools>,
        world: World,
        targets: Targets,
        scheduler: Scheduler,
    }
    static CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
    struct Reader;
    impl Mod for Reader {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *CONTEXT.lock().unwrap() = Some(Context {
                tools: Arc::new(crate::Mcp::init(load)?.tools),
                world: load.world()?,
                targets: load.interaction_targets()?,
                scheduler: load.scheduler()?,
            });
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Reader, "id = \"test.prompt-lifecycle\"\nside = \"server\"");
    fn start() -> (enshwick::test::Installed, Context) {
        let fake = enshwick::test::FakeHost::new()
            .world()
            .interaction_targets();
        let used_item = fake.hash(facts::USED_ITEM.id()).unwrap();
        let ty = fake.hash(facts::SELECTED_TARGETS.id()).unwrap();
        let mut bytes = vec![0; fake.size(facts::SELECTED_TARGETS.id()).unwrap()];
        let stride = fake.size(facts::TARGET_INFO.id()).unwrap();
        let offset = fake.offset(facts::TARGET_INFO_ENTITY.id()).unwrap();
        bytes[offset..offset + 4].copy_from_slice(&77u32.to_le_bytes());
        let at = stride * 2 + offset;
        bytes[at..at + 4].copy_from_slice(&512u32.to_le_bytes());
        let mut player = PlayerInfo {
            header: Header::of::<PlayerInfo>(),
            slot: 0,
            entity: Entity(40),
            key: PlayerKey::new(7),
            permissions: 0,
            name_len: 10,
            name: [0; PlayerInfo::NAME_MAX],
        };
        player.name[..10].copy_from_slice(b"Researcher");
        let loader = fake
            .player(player)
            .component(Entity(40), ty, &bytes)
            .component(Entity(512), used_item, &41u32.to_le_bytes())
            .install();
        // SAFETY: generated exports belong to this isolated fake-only reader.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let context = CONTEXT.lock().unwrap().as_ref().unwrap().clone();
        (loader, context)
    }
    fn assert_retained(context: &Context) {
        let players = context.world.players();
        assert_eq!(players.len(), 1);
        assert_eq!(players[0].entity, Entity(40));
        assert_eq!(players[0].key, PlayerKey::new(7));
        assert_eq!(
            context.targets.target(Entity(40)).unwrap(),
            Some(Entity(512))
        );
        assert_eq!(
            context.world.used_item(Entity(512)).map(|item| item.0),
            Some(41)
        );
    }
    #[test]
    fn queued_prompt_reads_refuse_changed_lifecycle_with_retained_player_target() {
        for phase in ["unchanged", "reload", "shutdown"] {
            let (loader, context) = start();
            let _ = loader.tick(Duration::from_millis(1));
            let scheduler = context.scheduler;
            assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
            let tools = context.tools.clone();
            let worker = thread::spawn(move || {
                tools.call(
                    "player_interaction_target",
                    json!({"player":"7"}).as_object().unwrap(),
                )
            });
            // Pinned SDK Debug observes enqueue only in this isolated queue.
            let until = Instant::now() + Duration::from_secs(2);
            while format!("{scheduler:?}") != "Scheduler { pending: true }" {
                assert!(!worker.is_finished(), "{phase}: read did not queue");
                assert!(Instant::now() < until, "{phase}: queue barrier not reached");
                thread::yield_now();
            }
            match phase {
                "reload" => assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0),
                "shutdown" => assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0),
                _ => {}
            }
            assert_retained(&context);
            let until = Instant::now() + Duration::from_secs(2);
            while !worker.is_finished() && Instant::now() < until {
                let _ = loader.tick(Duration::from_millis(1));
                thread::yield_now();
            }
            let result = worker.join().unwrap();
            if phase == "unchanged" {
                let value = result.unwrap();
                assert_eq!(value["player"]["entity"], 40);
                assert_eq!(value["observation"]["state"], "observed");
                assert_eq!(value["observation"]["target"]["entity"], 512);
                assert_eq!(value["observation"]["target"]["item_id"], 41);
            } else {
                let error =
                    result.expect_err("queued changed-context prompt read returned an observation");
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
    fn shutdown_before_prompt_admission_refuses_retained_player_target() {
        let (loader, context) = start();
        assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
        assert_retained(&context);
        assert!(
            context
                .tools
                .call(
                    "player_interaction_target",
                    json!({"player":"7"}).as_object().unwrap()
                )
                .unwrap_err()
                .contains("active")
        );
    }
}
