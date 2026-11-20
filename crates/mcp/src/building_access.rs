//! Explicit-box permission observations, not item placement or save verification.
use enshwick::world::BuildRefusal;
use policy::mcp::Tool;
use serde_json::{Value, json};

pub(crate) fn tool() -> Tool {
    Tool {
        name: "building_access",
        description: "Read-only native permission/persistence observations for explicit ordered site corners, each side >0 and <=64 metres. Requires active MCP world lifecycle at admission and execution with unchanged epoch. Optional player name/account key must match exactly one connected actor; omitted player leaves build rights unknown. No item footprint, collision, terrain contact, save/reload proof or write authority. Errors remain unknown, not false.",
        schema: policy::building_access::schema(),
    }
}

pub(crate) fn persistence(result: Result<bool, String>) -> Value {
    match result {
        Ok(value) => json!({"state":"observed","kept_by_zone_rule":value}),
        Err(error) => json!({"state":"unknown","kept_by_zone_rule":null,"error":error}),
    }
}

pub(crate) fn rights(result: Option<Result<(), BuildRefusal>>) -> Value {
    match result {
        None => json!({"state":"not_requested","allowed":null}),
        Some(Ok(())) => json!({"state":"observed","allowed":true}),
        Some(Err(BuildRefusal::Unchecked(error))) => {
            json!({"state":"unknown","allowed":null,"error":error.to_string()})
        }
        Some(Err(error)) => json!({"state":"refused","allowed":false,"reason":error.to_string()}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{WorldError, world::PlayerPermissions};
    #[test]
    fn missing_and_unchecked_never_become_negative_observations() {
        assert_eq!(persistence(Ok(false))["kept_by_zone_rule"], false);
        assert!(persistence(Err("unavailable".into()))["kept_by_zone_rule"].is_null());
        assert_eq!(rights(Some(Ok(())))["allowed"], true);
        assert_eq!(
            rights(Some(Err(BuildRefusal::NoPermission(
                PlayerPermissions::CanEditBase
            ))))["allowed"],
            false
        );
        assert_eq!(
            rights(Some(Err(BuildRefusal::Unchecked(WorldError::Unavailable))))["state"],
            "unknown"
        );
        assert!(rights(None)["allowed"].is_null());
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use crate::tools::GameTools;
    use enshwick::{
        Entity, Load, Mod, Off, PlayerKey,
        abi::{Header, PlayerInfo, Status},
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
        scheduler: Scheduler,
    }
    static CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
    struct Reader;
    impl Mod for Reader {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *CONTEXT.lock().unwrap() = Some(Context {
                tools: Arc::new(crate::Mcp::init(load)?.tools),
                world: load.world()?,
                scheduler: load.scheduler()?,
            });
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Reader, "id = \"test.access-lifecycle\"\nside = \"server\"");
    fn start() -> (enshwick::test::Installed, Context) {
        let mut player = PlayerInfo {
            header: Header::of::<PlayerInfo>(),
            slot: 0,
            entity: Entity(40),
            key: PlayerKey::new(7),
            permissions: PlayerInfo::CAN_EDIT_WORLD | PlayerInfo::CAN_EDIT_BASE,
            name_len: 10,
            name: [0; PlayerInfo::NAME_MAX],
        };
        player.name[..10].copy_from_slice(b"Researcher");
        let loader = enshwick::test::FakeHost::new()
            .world()
            .player(player)
            .building_zone([0.; 3], [40.; 3])
            .building_area([0.; 3], [1000.; 3])
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
    fn args(player: bool) -> Value {
        let mut value = json!({"x1":1.,"y1":1.,"z1":1.,"x2":5.,"y2":5.,"z2":5.});
        if player {
            value["player"] = json!("7");
        }
        value
    }
    fn assert_retained(context: &Context) {
        assert_eq!(context.world.persists([1.; 3], [5.; 3]), Ok(true));
        let players = context.world.players();
        assert_eq!(players.len(), 1);
        assert_eq!(players[0].entity, Entity(40));
        assert_eq!(players[0].key, PlayerKey::new(7));
        assert_eq!(
            context.world.can_build(&players[0], [1.; 3], [5.; 3]),
            Ok(())
        );
    }
    fn queued(player: bool) {
        for phase in ["unchanged", "reload", "shutdown"] {
            let (loader, context) = start();
            let _ = loader.tick(Duration::from_millis(1));
            let scheduler = context.scheduler;
            assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
            let tools = context.tools.clone();
            let worker = thread::spawn(move || {
                tools.call("building_access", args(player).as_object().unwrap())
            });
            // Pinned SDK Debug is an enqueue barrier only for this isolated queue.
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
            assert!(loader.transactions().is_empty());
            if phase == "unchanged" {
                let value = result.unwrap();
                assert_eq!(value["persistence"]["state"], "observed");
                assert_eq!(value["persistence"]["kept_by_zone_rule"], true);
                if player {
                    assert_eq!(value["player"]["entity"], 40);
                    assert_eq!(value["build_rights"]["state"], "observed");
                    assert_eq!(value["build_rights"]["allowed"], true);
                } else {
                    assert_eq!(value["build_rights"]["state"], "not_requested");
                    assert!(value["build_rights"]["allowed"].is_null());
                }
                assert_eq!(value["world_mutated"], false);
            } else {
                let error =
                    result.expect_err("queued changed-context access read returned an observation");
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
    fn queued_world_access_refuses_changed_lifecycle_with_retained_zone() {
        queued(false);
    }
    #[test]
    fn queued_player_access_refuses_changed_lifecycle_with_retained_zone() {
        queued(true);
    }
    #[test]
    fn shutdown_before_access_admission_refuses_both_modes_with_retained_zone() {
        for player in [false, true] {
            let (loader, context) = start();
            assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
            assert_retained(&context);
            assert!(
                context
                    .tools
                    .call("building_access", args(player).as_object().unwrap())
                    .unwrap_err()
                    .contains("active")
            );
            assert!(loader.transactions().is_empty());
        }
    }
}
