//! Published server active-buff observations; never apply or remove a buff.
use enshwick::{
    Entity,
    buffs::{BuffSlotType, Buffs},
};
use serde_json::{Value, json};

pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "entity_buffs_read",
        description: "Read active buffs for one session-only server entity on the game thread. Requires active MCP world lifecycle at admission and execution with unchanged epoch. At most 64 output rows, default 32; SDK first retrieves the full entity list. Read errors remain errors, not empty/healthy. Preserve type and buff-entity IDs, SDK slot/optional level and exact game-time duration nanoseconds as strings. SDK maps negative durations to zero and zero levels to null. No type-name lookup, complete physical effects, resting/safety, rendered-UI or mutation claim.",
        schema: policy::buff_read::schema(),
    }
}

pub(crate) fn read(buffs: Buffs, entity: u32, limit: usize) -> Result<Value, String> {
    let values = buffs
        .of(Entity(entity))
        .map_err(|e| format!("active buffs read failed: {e}"))?;
    let rows: Vec<_> = values
        .iter()
        .take(limit)
        .map(|buff| {
            let (kind, other) = match buff.slot {
                BuffSlotType::Food => ("food", None),
                BuffSlotType::Buff => ("buff", None),
                BuffSlotType::Debuff => ("debuff", None),
                BuffSlotType::Rested => ("rested", None),
                BuffSlotType::TerrainDebuff => ("terrain_debuff", None),
                BuffSlotType::Other(value) => ("other", Some(value)),
                _ => ("unknown", None),
            };
            json!({"type_id":buff.type_id.0,"buff_entity":buff.buff_entity_id.0,
            "slot":{"kind":kind,"other_sdk_value":other},"level":buff.level,
            "lifetime_game_ns":buff.life_time.as_nanos().to_string(),
            "remaining_game_ns":buff.remaining.as_nanos().to_string()})
        })
        .collect();
    Ok(
        json!({"source":"published_server_entity_buffs","entity":entity,
        "state":"observed","total_returned_by_sdk":values.len(),
        "truncated":values.len()>limit,"buffs":rows,
        "duration_projection":"SDK nonnegative Duration in game time; negative native values become zero",
        "level_projection":"SDK maps native zero to null",
        "rendered_ui_verified":false,"effects_verified":false,"actor_safety_verified":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::{
        Load, Mod, Off,
        abi::{BuffTypeInfo, Header, Status},
        buffs::{Apply, BuffTypeId},
        schedule::Scheduler,
    };
    use std::{
        sync::{Mutex, mpsc},
        time::Duration,
    };
    static CONTEXT: Mutex<Option<(Buffs, Scheduler)>> = Mutex::new(None);
    struct Probe;
    impl Mod for Probe {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *CONTEXT.lock().unwrap() = Some((load.buffs(), load.scheduler()?));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Probe, "id = \"test.buff-read\"\nside = \"server\"");

    #[test]
    fn scheduled_reads_preserve_errors_empty_projection_unknown_slots_and_full_state() {
        let loader = enshwick::test::FakeHost::new().install();
        // SAFETY: these are this test module's framework-generated exports.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        let (buffs, scheduler) = CONTEXT.lock().unwrap().unwrap();
        let sample = |entity, limit| {
            let (tx, rx) = mpsc::channel();
            scheduler.next_tick(move || {
                tx.send(read(buffs, entity, limit)).unwrap();
            });
            assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
            let _ = loader.tick(Duration::from_millis(100));
            rx.try_recv().unwrap()
        };
        assert!(
            sample(1, 32)
                .unwrap_err()
                .starts_with("active buffs read failed:")
        );
        let slots = [
            BuffTypeInfo::FOOD,
            BuffTypeInfo::BUFF,
            BuffTypeInfo::DEBUFF,
            BuffTypeInfo::RESTED,
            BuffTypeInfo::TERRAIN,
            65534,
        ];
        let types: Vec<_> = (1..=69)
            .map(|id| BuffTypeInfo {
                header: Header::of::<BuffTypeInfo>(),
                id: BuffTypeId(id),
                slot: slots[(id as usize - 1) % slots.len()],
                flags: 0,
                name: 0,
                duration_ns: if id == 1 { -1 } else { i64::MAX },
            })
            .collect();
        loader.set_buffs(&types, &[Entity(1), Entity(2)]);
        let empty = sample(1, 32).unwrap();
        assert_eq!(empty["buffs"], json!([]));
        assert_eq!(empty["state"], "observed");
        for id in 1..=69 {
            buffs
                .apply(
                    Entity(1),
                    BuffTypeId(id),
                    if id == 2 {
                        Apply::new().level(3)
                    } else {
                        Apply::new()
                    },
                )
                .unwrap();
        }
        let baseline = loader.buffs_of(Entity(1));
        let value = sample(1, 64).unwrap();
        assert_eq!(value["total_returned_by_sdk"], 69);
        assert_eq!(value["buffs"].as_array().unwrap().len(), 64);
        assert_eq!(value["truncated"], true);
        assert_eq!(value["buffs"][0]["lifetime_game_ns"], "0");
        assert!(value["buffs"][0]["level"].is_null());
        assert_eq!(value["buffs"][1]["remaining_game_ns"], i64::MAX.to_string());
        assert_eq!(value["buffs"][1]["level"], 3);
        for (i, kind) in [
            "food",
            "buff",
            "debuff",
            "rested",
            "terrain_debuff",
            "other",
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(value["buffs"][i]["slot"]["kind"], *kind);
        }
        assert_eq!(value["buffs"][5]["slot"]["other_sdk_value"], 65534);
        assert_eq!(sample(1, 1).unwrap()["buffs"].as_array().unwrap().len(), 1);
        assert_eq!(loader.buffs_of(Entity(1)), baseline);
        assert_eq!(loader.buffs_of(Entity(2)), Some(Vec::new()));
        assert!(sample(u32::MAX, 1).is_err());
        for key in [
            "rendered_ui_verified",
            "effects_verified",
            "actor_safety_verified",
        ] {
            assert_eq!(value[key], false);
        }
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use crate::tools::GameTools;
    use enshwick::{
        Entity, Load, Mod, Off,
        abi::{BuffTypeInfo, Header, Status},
        buffs::{Apply, BuffTypeId, Buffs},
        schedule::Scheduler,
    };
    use policy::mcp::Tools;
    use serde_json::json;
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::{Duration, Instant},
    };
    static CONTEXT: Mutex<Option<(Arc<GameTools>, Buffs, Scheduler)>> = Mutex::new(None);
    struct Reader;
    impl Mod for Reader {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *CONTEXT.lock().unwrap() = Some((
                Arc::new(crate::Mcp::init(load)?.tools),
                load.buffs(),
                load.scheduler()?,
            ));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Reader, "id = \"test.buff-lifecycle\"\nside = \"server\"");
    fn start() -> (enshwick::test::Installed, Arc<GameTools>, Buffs, Scheduler) {
        let loader = enshwick::test::FakeHost::new().world().install();
        // SAFETY: generated exports belong to this isolated fake-only reader.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let (tools, buffs, scheduler) = CONTEXT.lock().unwrap().as_ref().unwrap().clone();
        loader.set_buffs(
            &[BuffTypeInfo {
                header: Header::of::<BuffTypeInfo>(),
                id: BuffTypeId(1),
                slot: BuffTypeInfo::BUFF,
                flags: 0,
                name: 0,
                duration_ns: 100_000_000_000,
            }],
            &[Entity(17)],
        );
        buffs
            .apply(Entity(17), BuffTypeId(1), Apply::new().level(3))
            .unwrap();
        (loader, tools, buffs, scheduler)
    }
    #[test]
    fn queued_buff_reads_refuse_changed_lifecycle_with_retained_data() {
        for phase in ["unchanged", "reload", "shutdown"] {
            let (loader, tools, buffs, scheduler) = start();
            let _ = loader.tick(Duration::from_millis(1));
            assert_eq!(format!("{scheduler:?}"), "Scheduler { pending: false }");
            let worker = thread::spawn(move || {
                tools.call(
                    "entity_buffs_read",
                    json!({"entity":17}).as_object().unwrap(),
                )
            });
            // Pinned SDK Debug observes queueing only, not execution or liveness.
            // This fixture has no competing scheduler tasks.
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
            let retained = buffs.of(Entity(17)).unwrap();
            assert_eq!(retained.len(), 1);
            assert_eq!(retained[0].level, Some(3));
            let until = Instant::now() + Duration::from_secs(2);
            while !worker.is_finished() && Instant::now() < until {
                let _ = loader.tick(Duration::from_millis(1));
                thread::yield_now();
            }
            let result = worker.join().unwrap();
            if phase == "unchanged" {
                let value = result.unwrap();
                assert_eq!(value["entity"], 17);
                assert_eq!(value["buffs"][0]["level"], 3);
            } else {
                let error =
                    result.expect_err("queued changed-context buff read returned an observation");
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
    fn shutdown_before_buff_admission_refuses_retained_entity_data() {
        let (loader, tools, buffs, _) = start();
        assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
        assert_eq!(buffs.of(Entity(17)).unwrap().len(), 1);
        assert!(
            tools
                .call(
                    "entity_buffs_read",
                    json!({"entity":17}).as_object().unwrap()
                )
                .unwrap_err()
                .contains("active")
        );
    }
}
