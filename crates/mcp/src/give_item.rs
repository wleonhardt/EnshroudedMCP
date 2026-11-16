//! Once-only caller-identified inventory grants; native commits are not read-back.
use crate::{
    construction::Construction,
    tools::{Game, GameTools, give_dispatch, give_gate},
};
use policy::{
    give_item as grant,
    write_gate::{self, PendingWrite},
};
use serde_json::{Map, Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    time::Instant,
};

#[derive(Debug)]
struct Entry {
    session: String,
    request: Value,
    result: Value,
}
#[derive(Clone, Debug)]
pub(crate) struct Grants {
    dir: PathBuf,
    lifecycle: Construction,
    operations: Arc<Mutex<BTreeMap<String, Entry>>>,
}
fn path(dir: &Path, operation: &str) -> PathBuf {
    dir.join("give-operations")
        .join(format!("grant-{operation}.json"))
}
fn reserve(dir: &Path, operation: &str, intent: &Value, count: usize) -> Result<(), String> {
    grant::capacity(count)?;
    let namespace = dir.join("give-operations");
    match fs::read_dir(&namespace) {
        Ok(entries) => {
            for (index, entry) in entries.enumerate() {
                entry.map_err(|e| e.to_string())?;
                grant::capacity(index + 1)?;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.to_string()),
    }
    let bytes = serde_json::to_vec(intent).map_err(|e| e.to_string())?;
    write_gate::journal_budget(0, bytes.len(), 0)?;
    fs::create_dir_all(namespace).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path(dir, operation))
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    let size = match fs::metadata(dir.join("journal.jsonl")) {
        Ok(m) if m.is_file() => m.len(),
        Ok(_) => return Err("grant journal must be a regular file".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
        Err(e) => return Err(e.to_string()),
    };
    grant::journal_headroom(size)?;
    GameTools::journal(
        Some(dir),
        "give_item",
        intent,
        write_gate::JOURNAL_RECORD_BYTES,
    )
}
fn resolve(
    game: Game,
    spec: &grant::Spec,
) -> Result<(enshwick::world::Player, enshwick::ItemId, Value), String> {
    let guid = enshwick::content::vanilla::item(&spec.item).map_err(|e| e.to_string())?;
    let actor = policy::inspection::unique_player(
        game.world
            .players()
            .into_iter()
            .filter(|p| p.key.0.to_string() == spec.player || p.is_named(&spec.player)),
    )?;
    let item = enshwick::ItemId::from_guid(guid);
    let plan = json!({"player":actor.name,"actor_key":actor.key.0.to_string(),"actor_entity":actor.entity.0,"item":spec.item,"item_id":item.0,"guid":guid.to_string(),"count":spec.count});
    Ok((actor, item, plan))
}
impl Grants {
    pub(crate) fn new(dir: PathBuf, lifecycle: Construction) -> Self {
        Self {
            dir,
            lifecycle,
            operations: Arc::default(),
        }
    }
    fn record(&self, operation: &str, mut value: Value) -> Result<Value, String> {
        if let Err(error) = GameTools::journal(
            Some(&self.dir),
            "give_item",
            &json!({"kind":"give_item_outcome","operation":operation,"result":value}),
            0,
        ) {
            value["journal_error"] = json!(error);
        }
        let mut operations = self.operations.lock().map_err(|_| "grant state poisoned")?;
        operations
            .get_mut(operation)
            .ok_or("grant identity missing")?
            .result = value.clone();
        Ok(value)
    }
    pub(crate) fn call(&self, game: Game, args: &Map<String, Value>) -> Result<Value, String> {
        let spec = grant::check(args)?;
        give_gate(spec.dry, Some(&self.dir), spec.confirm.as_deref())?;
        let admitted = self.lifecycle.active_session()?;
        let control = self.clone();
        let started = Instant::now();
        let pending = Arc::new(PendingWrite::default());
        let queued = pending.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        game.scheduler.next_tick(move || {
            if !queued.begin() {
                return;
            }
            let result = control.run(game, &spec, &admitted, started);
            let _ = tx.send(result);
        });
        rx.recv_timeout(grant::WAIT).map_err(|_|if pending.cancel(){"grant timed out before execution; queued request cancelled".into()}else{"grant started; outcome unknown; query the same operation dry and inspect inventory/journal, never blindly retry".to_owned()})?
    }
    fn run(
        &self,
        game: Game,
        spec: &grant::Spec,
        admitted: &str,
        started: Instant,
    ) -> Result<Value, String> {
        let session = self.lifecycle.active_session()?;
        policy::construction::check_session(admitted, &session)?;
        give_gate(spec.dry, Some(&self.dir), spec.confirm.as_deref())?;
        let request = grant::request(spec);
        {
            let operations = self.operations.lock().map_err(|_| "grant state poisoned")?;
            if let Some(entry) = operations.get(&spec.operation) {
                policy::construction::check_session(&entry.session, &session)?;
                if entry.request != request {
                    return Err("grant operation payload conflict; never reuse an identity".into());
                }
                return Ok(entry.result.clone());
            }
        }
        if path(&self.dir, &spec.operation).exists() {
            return if spec.dry {
                Ok(
                    json!({"operation":spec.operation,"state":"durable_outcome_unknown","automatic_retry":false,"inventory_readback_verified":false}),
                )
            } else {
                Err("durable grant identity exists; never redispatch".into())
            };
        }
        grant::capacity(
            self.operations
                .lock()
                .map_err(|_| "grant state poisoned")?
                .len(),
        )?;
        grant::elapsed(started.elapsed())?;
        let (_, _, plan) = resolve(game, spec)?;
        let preview = grant::preview(spec, &session, &plan);
        if spec.dry {
            return Ok(
                json!({"operation":spec.operation,"session":session,"dry":true,"state":"dry","preview":preview,"would_give":plan,"inventory_readback_verified":false}),
            );
        }
        grant::check_preview(spec.preview.as_deref(), &preview)?;
        let inventories = game
            .inventories
            .ok_or("inventory transactions unavailable")?;
        let value = json!({"operation":spec.operation,"session":session,"state":"not_submitted_identity_blocked","native_submitted":false,"given":null,"plan":plan,"automatic_retry":false,"inventory_readback_verified":false});
        let count = {
            let mut operations = self.operations.lock().map_err(|_| "grant state poisoned")?;
            let count = operations.len();
            operations.insert(
                spec.operation.clone(),
                Entry {
                    session: session.clone(),
                    request,
                    result: value,
                },
            );
            count
        };
        let intent = json!({"kind":"give_item_intent","operation":spec.operation,"session":session,"plan":plan});
        if let Err(error) = reserve(&self.dir, &spec.operation, &intent, count) {
            self.operations
                .lock()
                .map_err(|_| "grant state poisoned")?
                .get_mut(&spec.operation)
                .unwrap()
                .result["journal_error"] = json!(error);
            return Err("grant intent failed; identity blocked without submission".into());
        }
        let ready = (|| {
            policy::construction::check_session(&session, &self.lifecycle.active_session()?)?;
            let (actor, item, current) = resolve(game, spec)?;
            grant::check_preview(Some(&preview), &grant::preview(spec, &session, &current))?;
            Ok((actor, item))
        })();
        let submit=ready.and_then(|(actor,item)|give_dispatch(Some(&self.dir),spec.confirm.as_deref(),||{
            grant::elapsed(started.elapsed())?;
            let mut submitted=json!({"operation":spec.operation,"session":session,"state":"submitted_outcome_pending","native_submitted":true,"given":null,"plan":plan,"automatic_retry":false,"inventory_readback_verified":false});
            self.operations.lock().map_err(|_|"grant state poisoned")?.get_mut(&spec.operation).unwrap().result=submitted.clone();
            let callback=self.clone();let operation=spec.operation.clone();
            let _ticket=inventories.give(actor.entity,item,spec.count,move|native|{
                submitted["state"]=json!("native_outcome_reported");submitted["given"]=json!(native.is_committed());submitted["outcome"]=json!(format!("{:?}",native.done));
                if let Err(error)=callback.record(&operation,submitted){enshwick::warn!("grant outcome cache failed: {error}");}
            });
            Ok(())
        }).and_then(|result|result));
        if let Err(error) = submit {
            let value = json!({"operation":spec.operation,"session":session,"state":"not_submitted_identity_blocked","native_submitted":false,"given":false,"refusal":error,"automatic_retry":false,"inventory_readback_verified":false});
            return self.record(&spec.operation, value);
        }
        Ok(self
            .operations
            .lock()
            .map_err(|_| "grant state poisoned")?
            .get(&spec.operation)
            .unwrap()
            .result
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::abi::{Header, InventoryResult, PlayerInfo, Status};
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::Duration,
    };
    struct Folder(PathBuf, PathBuf);
    impl Folder {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../gamedata/tests");
            fs::create_dir_all(&base).unwrap();
            let base = base.canonicalize().unwrap();
            let dir = base.join(format!(
                "grant-replay-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&dir).unwrap();
            Self(dir, base)
        }
        fn enable(&self) {
            fs::write(self.0.join(write_gate::FLAG), []).unwrap();
        }
        fn records(&self) -> Vec<Value> {
            match fs::read_to_string(self.0.join("journal.jsonl")) {
                Ok(text) => text
                    .lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect(),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                Err(e) => panic!("{e}"),
            }
        }
    }
    impl Drop for Folder {
        fn drop(&mut self) {
            let dir = self.0.canonicalize().unwrap();
            assert!(dir.starts_with(&self.1) && dir != self.1);
            fs::remove_dir_all(dir).unwrap();
        }
    }
    fn actor(key: u64, entity: u32, label: &str) -> PlayerInfo {
        let mut name = [0; 128];
        name[..label.len()].copy_from_slice(label.as_bytes());
        PlayerInfo {
            header: Header::of::<PlayerInfo>(),
            slot: 0,
            entity: enshwick::Entity(entity),
            key: enshwick::PlayerKey::new(key),
            permissions: 0,
            name_len: label.len() as u32,
            name,
        }
    }
    static ADAPTER: Mutex<Option<(Grants, Game)>> = Mutex::new(None);
    struct Probe;
    impl enshwick::Mod for Probe {
        fn init(load: &mut enshwick::Load<'_>) -> Result<Self, enshwick::Off> {
            let control = Grants::new(load.mod_dir()?, Construction::new(load)?);
            let game = Game {
                world: load.world()?,
                native_knowledge: load.knowledge(),
                clock: load.clock(),
                weather: load.weather(),
                buffs: load.buffs(),
                game_settings: None,
                voxels: load.voxels(),
                scheduler: load.scheduler()?,
                used_item: None,
                static_used_item: None,
                inventories: Some(load.inventories()?),
                item_layout: None,
                network_health: None,
                factory_station: None,
                interaction_targets: None,
            };
            *ADAPTER.lock().unwrap() = Some((control, game));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.grant-replay"
side = "server""#
    );
    fn input(operation: &str) -> Value {
        json!({"operation":operation,"player":"10","item":"fixture_item"})
    }
    fn real(mut request: Value, preview: &Value) -> Value {
        request["dry"] = json!(false);
        request["confirm"] = json!("server-write");
        request["preview"] = preview["preview"].clone();
        request
    }
    #[test]
    fn durable_intents_are_exclusive_capacity_bounded_and_reserve_outcome_headroom() {
        let f = Folder::new();
        let intent = json!({"kind":"fixture"});
        reserve(&f.0, "con", &intent, 0).unwrap();
        let bytes = fs::read(path(&f.0, "con")).unwrap();
        assert!(reserve(&f.0, "con", &json!({"changed":true}), 1).is_err());
        assert_eq!(fs::read(path(&f.0, "con")).unwrap(), bytes);
        assert!(reserve(&f.0, "new", &intent, 64).is_err());
        assert!(!path(&f.0, "new").exists());
        let full = Folder::new();
        fs::create_dir(full.0.join("give-operations")).unwrap();
        for i in 0..64 {
            fs::write(path(&full.0, &format!("{i}")), b"{}").unwrap();
        }
        assert!(reserve(&full.0, "new", &intent, 0).is_err());
        assert!(!path(&full.0, "new").exists());
        let g = Folder::new();
        let size = (policy::construction::JOURNAL_MAX_BYTES - 2 * write_gate::JOURNAL_RECORD_BYTES
            + 1) as u64;
        let file = fs::File::create(g.0.join("journal.jsonl")).unwrap();
        file.set_len(size).unwrap();
        drop(file);
        assert!(reserve(&g.0, "headroom", &intent, 0).is_err());
        assert_eq!(fs::metadata(g.0.join("journal.jsonl")).unwrap().len(), size);
    }
    #[test]
    fn native_grants_reconcile_pending_and_late_outcomes_without_replay_or_stale_dispatch() {
        let f = Folder::new();
        f.enable();
        let loader = enshwick::test::FakeHost::new()
            .world()
            .inventory_requests()
            .items()
            .workshops()
            .vanilla_items(&[(enshwick::content::Guid([7; 16]), "fixture_item")])
            .player(actor(9, 7, "actor"))
            .player(actor(10, 8, "actor"))
            .player(actor(11, 9, "9"))
            .mod_dir(f.0.to_str().unwrap())
            .install();
        // SAFETY: this module's Probe generates the init export.
        assert_eq!(unsafe { loader.init(enshwick_mod_init) }, Status::OK);
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let (control, game) = ADAPTER.lock().unwrap().as_ref().unwrap().clone();
        let drive = |request: Value| {
            let adapter = control.clone();
            let worker =
                std::thread::spawn(move || adapter.call(game, request.as_object().unwrap()));
            let deadline = Instant::now() + Duration::from_secs(8);
            while !worker.is_finished() {
                assert!(Instant::now() < deadline);
                let _ = loader.tick(Duration::from_millis(50));
                std::thread::sleep(Duration::from_millis(1));
            }
            worker.join().unwrap()
        };
        let mut invalid = input("invalid");
        invalid["count"] = json!(-1);
        assert!(drive(invalid).is_err());
        for selector in ["actor", "9"] {
            let mut bad = input("ambiguous");
            bad["player"] = json!(selector);
            assert!(drive(bad).unwrap_err().contains("ambiguous"));
        }
        assert!(loader.inventory_requests().is_empty() && f.records().is_empty());
        let dry = input("once");
        let preview = drive(dry.clone()).unwrap();
        assert_eq!(preview["state"], "dry");
        assert!(!f.0.join("give-operations").exists());
        let request = real(dry.clone(), &preview);
        let cancelled = control.clone();
        let late = request.clone();
        assert!(
            std::thread::spawn(move || cancelled.call(game, late.as_object().unwrap()))
                .join()
                .unwrap()
                .unwrap_err()
                .contains("queued request cancelled")
        );
        let _ = loader.tick(Duration::from_millis(50));
        assert!(loader.inventory_requests().is_empty() && f.records().is_empty());
        // Discard the submission acknowledgement, then reconcile the original caller identity.
        let _ = drive(request.clone()).unwrap();
        let pending = drive(dry.clone()).unwrap();
        assert_eq!(pending["state"], "submitted_outcome_pending");
        assert!(pending["given"].is_null());
        assert_eq!(loader.inventory_requests().len(), 1);
        assert_eq!(f.records().len(), 1);
        assert_eq!(drive(request.clone()).unwrap(), pending);
        assert_eq!(loader.inventory_requests().len(), 1);
        let mut conflict = dry.clone();
        conflict["count"] = json!(2);
        assert!(drive(conflict).unwrap_err().contains("payload conflict"));
        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
        assert!(loader.finish_inventory(InventoryResult::COMMITTED, &[]));
        let committed = drive(dry.clone()).unwrap();
        assert_eq!(committed["given"], true);
        assert_eq!(committed["inventory_readback_verified"], false);
        assert_eq!(f.records().len(), 2);
        assert_eq!(
            f.records()[0]["result"]["operation"],
            f.records()[1]["result"]["operation"]
        );
        let reopened = Grants::new(f.0.clone(), control.lifecycle.clone());
        let unknown = grant::check(dry.as_object().unwrap()).unwrap();
        assert_eq!(
            reopened
                .run(
                    game,
                    &unknown,
                    &control.lifecycle.active_session().unwrap(),
                    Instant::now()
                )
                .unwrap()["state"],
            "durable_outcome_unknown"
        );
        f.enable();
        assert!(
            reopened
                .run(
                    game,
                    &grant::check(request.as_object().unwrap()).unwrap(),
                    &control.lifecycle.active_session().unwrap(),
                    Instant::now()
                )
                .is_err()
        );
        assert!(loader.inventory_requests().is_empty());
        let rollback = input("rollback");
        let p = drive(rollback.clone()).unwrap();
        drive(real(rollback.clone(), &p)).unwrap();
        assert!(loader.finish_inventory(InventoryResult::ROLLED_BACK, &[]));
        assert_eq!(drive(rollback).unwrap()["given"], false);
        assert!(loader.inventory_requests().is_empty());
        let stale = input("stale");
        let stale_preview = drive(stale.clone()).unwrap();
        let late = input("late-outcome");
        let p = drive(late.clone()).unwrap();
        drive(real(late.clone(), &p)).unwrap();
        let original_session = p["session"].clone();
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        assert!(loader.finish_inventory(InventoryResult::COMMITTED, &[]));
        assert_eq!(
            f.records().last().unwrap()["result"]["result"]["session"],
            original_session
        );
        assert!(drive(late).is_err());
        assert!(drive(dry).is_err());
        assert!(drive(real(stale, &stale_preview)).is_err());
        assert!(!path(&f.0, "stale").exists());
        assert!(loader.inventory_requests().is_empty());
        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
    }
    #[test]
    fn intent_failure_and_callback_audit_failure_block_retries_and_keep_status_explicit() {
        let f = Folder::new();
        f.enable();
        let loader = enshwick::test::FakeHost::new()
            .world()
            .inventory_requests()
            .items()
            .workshops()
            .vanilla_items(&[(enshwick::content::Guid([7; 16]), "fixture_item")])
            .player(actor(10, 8, "actor"))
            .mod_dir(f.0.to_str().unwrap())
            .install();
        // SAFETY: this module's Probe generates the init export.
        assert_eq!(unsafe { loader.init(enshwick_mod_init) }, Status::OK);
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let (control, game) = ADAPTER.lock().unwrap().as_ref().unwrap().clone();
        let run = |request: Value| {
            control.run(
                game,
                &grant::check(request.as_object().unwrap()).unwrap(),
                &control.lifecycle.active_session().unwrap(),
                Instant::now(),
            )
        };
        let dry = input("blocked");
        let preview = run(dry.clone()).unwrap();
        let request = real(dry.clone(), &preview);
        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
        assert!(run(request.clone()).is_err());
        assert!(!path(&f.0, "blocked").exists());
        f.enable();
        fs::create_dir(f.0.join("journal.jsonl")).unwrap();
        assert!(run(request).is_err());
        assert!(loader.inventory_requests().is_empty());
        assert!(run(dry).unwrap().get("journal_error").is_some());
        fs::remove_dir(f.0.join("journal.jsonl")).unwrap();
        let dry = input("audit");
        let preview = run(dry.clone()).unwrap();
        let request = real(dry.clone(), &preview);
        run(request).unwrap();
        assert_eq!(loader.inventory_requests().len(), 1);
        fs::remove_file(f.0.join("journal.jsonl")).unwrap();
        fs::create_dir(f.0.join("journal.jsonl")).unwrap();
        assert!(loader.finish_inventory(InventoryResult::COMMITTED, &[]));
        let observed = run(dry.clone()).unwrap();
        assert_eq!(observed["given"], true);
        assert!(observed.get("journal_error").is_some());
        assert_eq!(run(dry).unwrap(), observed);
        assert!(loader.inventory_requests().is_empty());
        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
    }
}
