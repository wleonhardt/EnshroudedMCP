//! Positive-only administrative health restoration, scheduled on the game thread.
use crate::{
    construction::Construction,
    tools::{Game, GameTools, give_gate},
};
use enshwick::{Load, Off};
use policy::{
    health_recovery::{self, Context, Spec},
    write_gate::{self, PendingWrite},
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::Duration,
};

#[derive(Clone, Debug)]
pub(crate) struct Recovery {
    construction: Construction,
    generation: Arc<AtomicU64>,
    dir: PathBuf,
}

impl Recovery {
    pub(crate) fn new(load: &mut Load<'_>, construction: Construction) -> Result<Self, Off> {
        let generation = Arc::new(AtomicU64::new(0));
        let changed = generation.clone();
        load.on::<enshwick::events::PlayerJoined>(move |_| {
            changed.fetch_add(1, Ordering::SeqCst);
        })?;
        let changed = generation.clone();
        load.on::<enshwick::events::PlayerLeft>(move |_| {
            changed.fetch_add(1, Ordering::SeqCst);
        })?;
        let changed = generation.clone();
        load.on::<enshwick::events::PlayerDied>(move |_| {
            changed.fetch_add(1, Ordering::SeqCst);
        })?;
        Ok(Self {
            construction,
            generation,
            dir: load.mod_dir()?,
        })
    }

    fn context(&self, game: Game, player: &str) -> Result<Context, String> {
        let session = self.construction.active_session()?;
        let players = game.world.players();
        health_recovery::check_roster(players.len())?;
        let actor = policy::inspection::unique_player(
            players
                .into_iter()
                .filter(|p| p.key.0.to_string() == player || p.is_named(player)),
        )?;
        let health = game
            .world
            .vitals(actor.entity)
            .ok_or("player vitals unavailable")?
            .health;
        let context = Context {
            session,
            generation: self.generation.load(Ordering::SeqCst),
            account: actor.key.0,
            entity: actor.entity.0,
            current: health.current,
            max: health.max,
        };
        health_recovery::preview(&context)?;
        Ok(context)
    }

    pub(crate) fn call(&self, game: Game, spec: Spec) -> Result<Value, String> {
        give_gate(spec.dry, Some(&self.dir), spec.confirm.as_deref())?;
        let admitted = self.construction.active_session()?;
        let this = self.clone();
        let pending = Arc::new(PendingWrite::default());
        let queued = pending.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        game.scheduler.next_tick(move || {
            if !queued.begin() {
                return;
            }
            let result = (|| {
                policy::construction::check_session(
                    &admitted,
                    &this.construction.active_session()?,
                )?;
                run_with(
                    &spec,
                    Some(&this.dir),
                    || this.context(game, &spec.player),
                    |context| {
                        game.world
                            .set_health(enshwick::Entity(context.entity), context.max)
                            .map(|_| ())
                            .map_err(|e| e.to_string())
                    },
                )
            })();
            let _ = tx.send(result);
        });
        rx.recv_timeout(Duration::from_secs(5)).map_err(|_| {
            if pending.cancel() { "health recovery timed out before execution; queued request cancelled".to_owned() }
            else { "health recovery started; outcome unknown; inspect players and journal, do not blindly retry".to_owned() }
        })?
    }
}

fn run_with(
    spec: &Spec,
    dir: Option<&Path>,
    mut read: impl FnMut() -> Result<Context, String>,
    mut change: impl FnMut(&Context) -> Result<(), String>,
) -> Result<Value, String> {
    give_gate(spec.dry, dir, spec.confirm.as_deref())?;
    let before = read()?;
    let preview = health_recovery::preview(&before)?;
    let mut result = json!({"state":"dry","preview":preview,"before":{"current":before.current,"max":before.max},
        "requested_health":before.max,"native_request_accepted":false,"readback_equal":false,
        "automatic_retry":false,"invulnerability":false,"native_undo":false,
        "scope":"server administrative health only; not item use or combat safety"});
    if spec.dry {
        return Ok(result);
    }
    health_recovery::check_preview(spec.preview.as_deref().unwrap_or(""), &before)?;
    if before.current == before.max {
        result["state"] = json!("already_full");
        return Ok(result);
    }
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let request = format!(
        "{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    GameTools::journal(
        dir,
        "player_restore_health",
        &json!({"kind":"health_recovery_intent","request":request,"preview":preview}),
        write_gate::JOURNAL_RECORD_BYTES,
    )?;
    let ready = (|| {
        give_gate(false, dir, spec.confirm.as_deref())?;
        health_recovery::check_preview(&preview, &read()?)?;
        give_gate(false, dir, spec.confirm.as_deref())
    })();
    if let Err(error) = ready {
        GameTools::journal(
            dir,
            "player_restore_health",
            &json!({"kind":"health_recovery_outcome","request":request,"state":"not_submitted"}),
            0,
        )?;
        return Err(error);
    }
    let native = change(&before);
    let observed = read().ok();
    let equal = observed
        .as_ref()
        .is_some_and(|after| health_recovery::restored(&before, after));
    result["request"] = json!(request);
    result["native_request_accepted"] = json!(native.is_ok());
    result["readback_equal"] = json!(equal);
    result["after"] = json!(observed.map(|c| json!({"current":c.current,"max":c.max})));
    result["state"] = json!(if native.is_err() {
        "command_error_unknown"
    } else if equal {
        "restored_readback"
    } else {
        "readback_unverified"
    });
    GameTools::journal(dir,"player_restore_health",&json!({"kind":"health_recovery_outcome","request":request,"result":result}),0)
        .map_err(|e|format!("health may have changed; outcome journal failed; inspect state, do not blindly retry: {e}"))?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, fs};
    static ADAPTER: std::sync::Mutex<Option<(Recovery, Game)>> = std::sync::Mutex::new(None);
    struct Probe;
    impl enshwick::Mod for Probe {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            let game = Game {
                world: load.world()?,
                native_knowledge: load.knowledge(),
                clock: load.clock(),
                game_settings: None,
                voxels: load.voxels(),
                scheduler: load.scheduler()?,
                used_item: None,
                static_used_item: None,
                inventories: None,
                item_layout: None,
                network_health: None,
                interaction_targets: None,
            };
            let construction = Construction::new(load)?;
            *ADAPTER.lock().unwrap() = Some((Recovery::new(load, construction)?, game));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.health-recovery"
side = "server""#
    );
    fn context() -> Context {
        Context {
            session: "test".into(),
            generation: 1,
            account: 2,
            entity: 3,
            current: 17,
            max: 100,
        }
    }
    fn spec(dry: bool) -> Spec {
        Spec {
            player: "actor".into(),
            dry,
            preview: Some(health_recovery::preview(&context()).unwrap()),
            confirm: Some("server-write".into()),
        }
    }
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target")
                .join(format!(
                    "health-recovery-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            fs::create_dir(&path).unwrap();
            fs::write(path.join(write_gate::FLAG), []).unwrap();
            Self(path)
        }
        fn records(&self) -> Vec<Value> {
            fs::read_to_string(self.0.join("journal.jsonl"))
                .unwrap()
                .lines()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn dry_and_full_never_submit_or_journal_and_stale_preview_refuses() {
        let dry = run_with(
            &spec(true),
            None,
            || Ok(context()),
            |_| panic!("dry changed health"),
        )
        .unwrap();
        assert_eq!(dry["state"], "dry");
        let f = Fixture::new();
        let full = Context {
            current: 100,
            ..context()
        };
        let mut request = spec(false);
        request.preview = Some(health_recovery::preview(&full).unwrap());
        assert_eq!(
            run_with(
                &request,
                Some(&f.0),
                || Ok(full.clone()),
                |_| panic!("full changed health")
            )
            .unwrap()["state"],
            "already_full"
        );
        assert!(
            run_with(
                &spec(false),
                Some(&f.0),
                || Ok(full.clone()),
                |_| panic!("stale dispatched")
            )
            .is_err()
        );
        assert!(!f.0.join("journal.jsonl").exists());
        assert!(
            run_with(
                &spec(false),
                None,
                || panic!("read before gate"),
                |_| panic!("missing flag dispatched")
            )
            .is_err()
        );
    }
    #[test]
    fn durable_intent_precedes_mutation_and_outcome_matches_independent_read() {
        let f = Fixture::new();
        let current = Cell::new(17);
        let calls = Cell::new(0);
        let result = run_with(
            &spec(false),
            Some(&f.0),
            || {
                Ok(Context {
                    current: current.get(),
                    ..context()
                })
            },
            |before| {
                assert_eq!(f.records().len(), 1);
                assert_eq!(f.records()[0]["result"]["kind"], "health_recovery_intent");
                assert_eq!(before.current, 17);
                calls.set(calls.get() + 1);
                current.set(before.max);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(result["state"], "restored_readback");
        assert_eq!(result["readback_equal"], true);
        let records = f.records();
        assert_eq!(records.len(), 2);
        assert_eq!(
            records[0]["result"]["request"],
            records[1]["result"]["request"]
        );
    }
    #[test]
    fn journal_failure_and_changed_admission_prevent_native_write() {
        let f = Fixture::new();
        fs::create_dir(f.0.join("journal.jsonl")).unwrap();
        assert!(
            run_with(
                &spec(false),
                Some(&f.0),
                || Ok(context()),
                |_| panic!("journal failure dispatched")
            )
            .is_err()
        );
        fs::remove_dir(f.0.join("journal.jsonl")).unwrap();
        let reads = Cell::new(0);
        assert!(
            run_with(
                &spec(false),
                Some(&f.0),
                || {
                    reads.set(reads.get() + 1);
                    if reads.get() == 1 {
                        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
                    }
                    Ok(context())
                },
                |_| panic!("revoked flag dispatched")
            )
            .is_err()
        );
        assert_eq!(f.records().len(), 2);
        fs::write(f.0.join(write_gate::FLAG), []).unwrap();
        let reads = Cell::new(0);
        assert!(
            run_with(
                &spec(false),
                Some(&f.0),
                || {
                    reads.set(reads.get() + 1);
                    Ok(Context {
                        generation: reads.get(),
                        ..context()
                    })
                },
                |_| panic!("changed actor dispatched")
            )
            .is_err()
        );
        assert_eq!(f.records().len(), 4);
        let reads = Cell::new(0);
        assert!(
            run_with(
                &spec(false),
                Some(&f.0),
                || {
                    reads.set(reads.get() + 1);
                    if reads.get() == 2 {
                        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
                    }
                    Ok(context())
                },
                |_| panic!("flag removed during final observation dispatched")
            )
            .is_err()
        );
    }
    #[test]
    fn native_error_unavailable_and_mismatched_readback_never_claim_restoration() {
        for case in 0..3 {
            let f = Fixture::new();
            let reads = Cell::new(0);
            let result = run_with(
                &spec(false),
                Some(&f.0),
                || {
                    reads.set(reads.get() + 1);
                    if case == 1 && reads.get() == 3 {
                        return Err("unavailable".into());
                    }
                    Ok(context())
                },
                |_| {
                    if case == 0 {
                        Err("native error".into())
                    } else {
                        Ok(())
                    }
                },
            )
            .unwrap();
            assert_eq!(result["readback_equal"], false);
            assert_ne!(result["state"], "restored_readback");
            assert_eq!(f.records().len(), 2);
        }
    }
    #[test]
    fn fake_host_scheduled_restore_cancels_late_work_and_invalidates_actor_preview() {
        use enshwick::Entity;
        use enshwick::abi::{Header, PlayerInfo, Status};
        let f = Fixture::new();
        let mut health = 10u32.to_le_bytes().to_vec();
        health.extend(8u16.to_le_bytes());
        health.extend([1, 8]);
        health.extend(
            [17i32, 0, 0, 0, 0, 0, 0, 100]
                .into_iter()
                .flat_map(i32::to_le_bytes),
        );
        let mut name = [0; 128];
        name[..5].copy_from_slice(b"actor");
        let loader = enshwick::test::FakeHost::new()
            .world()
            .mod_dir(f.0.to_str().unwrap())
            .type_hash("attribute_set.health", 1)
            .type_hash("attribute_set.stamina", 2)
            .type_hash("attribute_set.mana", 3)
            .type_hash("attribute_set.level", 4)
            .type_hash("attribute_set.experience", 5)
            .constant("ecs.level_data_offset", 4)
            .constant("ecs.level_flags", 6)
            .constant("ecs.attribute_count", 7)
            .constant("attribute.health", 10)
            .constant("attribute.health_max_slot", 7)
            .constant("attribute.stamina", 20)
            .constant("attribute.stamina_max_slot", 2)
            .constant("attribute.mana", 30)
            .constant("attribute.mana_max_slot", 1)
            .constant("attribute.level", 40)
            .constant("attribute.experience", 50)
            .constant("attribute.experience_needed_slot", 2)
            .component(Entity(7), 1, &health)
            .template(Entity(7), enshwick::content::Guid([7; 16]))
            .player(PlayerInfo {
                header: Header::of::<PlayerInfo>(),
                slot: 0,
                entity: Entity(7),
                key: enshwick::PlayerKey::new(9),
                permissions: 0,
                name_len: 5,
                name,
            })
            .install();
        // SAFETY: exports are generated by this module's declared test probe.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            Status::OK
        );
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let (recovery, game) = ADAPTER.lock().unwrap().as_ref().unwrap().clone();
        let request =
            health_recovery::check(json!({"player":"actor"}).as_object().unwrap()).unwrap();
        let drive = |request: Spec| {
            let adapter = recovery.clone();
            let worker = std::thread::spawn(move || adapter.call(game, request));
            let deadline = std::time::Instant::now() + Duration::from_secs(2);
            while !worker.is_finished() {
                assert!(std::time::Instant::now() < deadline);
                let _ = loader.tick(Duration::from_millis(50));
                std::thread::sleep(Duration::from_millis(1));
            }
            worker.join().unwrap()
        };
        let dry = drive(request.clone()).unwrap();
        assert_eq!(dry["state"], "dry");
        assert!(!f.0.join("journal.jsonl").exists());
        let mut real = request;
        real.dry = false;
        real.confirm = Some("server-write".into());
        real.preview = Some(dry["preview"].as_str().unwrap().into());
        let late = recovery.clone();
        let timed = real.clone();
        assert!(
            std::thread::spawn(move || late.call(game, timed))
                .join()
                .unwrap()
                .unwrap_err()
                .contains("queued request cancelled")
        );
        let _ = loader.tick(Duration::from_millis(50));
        assert_eq!(game.world.vitals(Entity(7)).unwrap().health.current, 17);
        assert!(!f.0.join("journal.jsonl").exists());
        let outcome = drive(real.clone()).unwrap();
        assert_eq!(outcome["state"], "restored_readback");
        assert_eq!(game.world.vitals(Entity(7)).unwrap().health.current, 100);
        assert_eq!(f.records().len(), 2);
        let fresh = drive(Spec {
            dry: true,
            ..real.clone()
        })
        .unwrap();
        real.preview = Some(fresh["preview"].as_str().unwrap().into());
        let left = enshwick::abi::PlayerData {
            header: Header::of::<enshwick::abi::PlayerData>(),
            slot: 0,
            entity: Entity(7),
            key: enshwick::PlayerKey::new(9),
        };
        assert!(loader.raise::<enshwick::events::PlayerLeft>(&left) > 0);
        assert!(drive(real).unwrap_err().contains("preview"));
        assert_eq!(f.records().len(), 2);
    }
}
