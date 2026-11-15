//! Deliberate native save requests; acceptance is not completed-save proof.
use crate::{construction::Construction, tools::GameTools};
use enshwick::{schedule::Scheduler, world::World};
use policy::{
    server_save as save,
    write_gate::{self, PendingWrite},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};

#[derive(Default, Debug)]
struct State {
    operations: BTreeMap<String, (String, Value)>,
}
#[derive(Clone, Debug)]
pub(crate) struct SaveControl {
    dir: PathBuf,
    lifecycle: Construction,
    state: Arc<Mutex<State>>,
}
pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "server_save",
        description: "ADMIN WRITE: ask the current server world to save through the published native API. Dry by default; operation identity required. Real request requires preview session, empty SDK player-character roster, fresh flag and confirm server-write. Roster is not a connection lock. Existing identities return retained status and never redispatch. A save already in progress counts as asked. Native acceptance does not prove completed save or exact persisted content; no automatic retry, undo, backup, save hold, shutdown or client control.",
        schema: save::schema(),
    }
}
fn gate(spec: &save::Spec, dir: &Path) -> Result<(), String> {
    write_gate::check(
        spec.dry,
        true,
        write_gate::flag_age(&dir.join(write_gate::FLAG)),
        spec.confirm.as_deref(),
    )
}
fn path(dir: &Path, operation: &str) -> PathBuf {
    dir.join("server-save")
        .join(format!("save-{operation}.json"))
}
fn capacity(dir: &Path, count: usize) -> Result<(), String> {
    save::capacity(count)?;
    match fs::read_dir(dir.join("server-save")) {
        Ok(entries) => {
            for (index, entry) in entries.enumerate() {
                entry.map_err(|e| e.to_string())?;
                save::capacity(index + 1)?;
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
fn reserve(dir: &Path, operation: &str, intent: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(intent).map_err(|e| e.to_string())?;
    write_gate::journal_budget(0, bytes.len(), 0)?;
    fs::create_dir_all(dir.join("server-save")).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path(dir, operation))
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())
}
fn outcome(dir: &Path, operation: &str, value: &mut Value) {
    if let Err(error) = GameTools::journal(
        Some(dir),
        "server_save",
        &json!({"kind":"server_save_outcome","operation":operation,"outcome":value}),
        0,
    ) {
        value["journal_error"] = json!(error);
    }
}
impl SaveControl {
    pub(crate) fn new(dir: PathBuf, lifecycle: Construction) -> Self {
        Self {
            dir,
            lifecycle,
            state: Arc::default(),
        }
    }
    pub(crate) fn call(
        &self,
        world: World,
        scheduler: Scheduler,
        spec: save::Spec,
    ) -> Result<Value, String> {
        gate(&spec, &self.dir)?;
        let admitted = self.lifecycle.active_session()?;
        let started = Instant::now();
        let pending = Arc::new(PendingWrite::default());
        let queued = pending.clone();
        let this = self.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        scheduler.next_tick(move || {
            if !queued.begin() {
                return;
            }
            let result = (|| {
                let active = this.lifecycle.active_session()?;
                policy::construction::check_session(&admitted, &active)?;
                let mut state = this.state.lock().map_err(|_| "save state poisoned")?;
                run_with(
                    &this.dir,
                    &mut state,
                    &spec,
                    &active,
                    || Ok((this.lifecycle.active_session()?, world.players().len())),
                    || world.save().map_err(|e| e.to_string()),
                    || started.elapsed(),
                )
            })();
            let _ = tx.send(result);
        });
        rx.recv_timeout(save::WAIT).map_err(|_|if pending.cancel(){"save timed out before execution; queued request cancelled".to_owned()}else{"save started; outcome unknown; inspect retained status, journal and independent save evidence, never blindly retry".to_owned()})?
    }
}
fn run_with(
    dir: &Path,
    state: &mut State,
    spec: &save::Spec,
    session: &str,
    mut read: impl FnMut() -> Result<(String, usize), String>,
    save: impl FnOnce() -> Result<(), String>,
    elapsed: impl Fn() -> Duration,
) -> Result<Value, String> {
    gate(spec, dir)?;
    if let Some((original, value)) = state.operations.get(&spec.operation) {
        policy::construction::check_session(original, session)?;
        if let Some(expected) = &spec.session {
            policy::construction::check_session(expected, original)?;
        }
        return Ok(value.clone());
    }
    if path(dir, &spec.operation).exists() {
        return if spec.dry {
            Ok(
                json!({"operation":spec.operation,"state":"durable_outcome_unknown","automatic_retry":false,"save_completed_verified":false,"causal_completion_verified":false}),
            )
        } else {
            Err("durable save identity exists; never redispatch".into())
        };
    }
    save::elapsed(elapsed())?;
    capacity(dir, state.operations.len())?;
    let (current, players) = read()?;
    policy::construction::check_session(session, &current)?;
    save::admit(spec, &current, players)?;
    let mut value = json!({"operation":spec.operation,"session":session,"state":"dry","observed_player_characters":players,"save_request_accepted":false,"save_dispatched":false,"save_completed_verified":false,"causal_completion_verified":false,"automatic_retry":false,"native_undo":false,"scope":"current server world; empty character roster is not a connection lock; preview does not prove native save availability; a save already in progress counts as asked; no backup, save hold or completed-save guarantee"});
    if spec.dry {
        return Ok(value);
    }
    let intent = json!({"kind":"server_save_intent","operation":spec.operation,"session":session});
    value["state"] = json!("not_issued_unknown");
    state
        .operations
        .insert(spec.operation.clone(), (session.into(), value));
    if let Err(error) = reserve(dir, &spec.operation, &intent).and_then(|()| {
        let size = match fs::metadata(dir.join("journal.jsonl")) {
            Ok(meta) if meta.is_file() => meta.len(),
            Ok(_) => return Err("save journal must be a regular file".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(error.to_string()),
        };
        save::journal_headroom(size)?;
        GameTools::journal(
            Some(dir),
            "server_save",
            &intent,
            write_gate::JOURNAL_RECORD_BYTES,
        )
    }) {
        state.operations.get_mut(&spec.operation).unwrap().1["journal_error"] = json!(error);
        return Err("save intent failed; identity blocked, no native request issued".into());
    }
    let ready = (|| {
        save::elapsed(elapsed())?;
        let (current, players) = read()?;
        policy::construction::check_session(session, &current)?;
        save::admit(spec, &current, players)?;
        gate(spec, dir)?;
        save::elapsed(elapsed())
    })();
    let value = &mut state.operations.get_mut(&spec.operation).unwrap().1;
    if let Err(error) = ready {
        value["state"] = json!("not_submitted");
        value["refusal"] = json!(error);
        outcome(dir, &spec.operation, value);
        return Err("save admission changed; identity blocked, no native request issued".into());
    }
    value["state"] = json!("issued_unknown");
    value["save_dispatched"] = json!(true);
    match save() {
        Ok(()) => {
            value["state"] = json!("save_accepted_completion_unverified");
            value["save_request_accepted"] = json!(true);
        }
        Err(error) => {
            value["state"] = json!("save_error_outcome_unknown");
            value["native_error"] = json!(error);
        }
    }
    outcome(dir, &spec.operation, value);
    Ok(value.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::sync::atomic::{AtomicU64, Ordering};
    struct Fixture(PathBuf, PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../gamedata/tests");
            fs::create_dir_all(&base).unwrap();
            let base = base.canonicalize().unwrap();
            let dir = base.join(format!(
                "save-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&dir).unwrap();
            Self(dir, base)
        }
        fn enable(&self) {
            fs::write(self.0.join(write_gate::FLAG), []).unwrap();
        }
        fn records(&self) -> usize {
            fs::read_to_string(self.0.join("journal.jsonl"))
                .unwrap()
                .lines()
                .count()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let p = self.0.canonicalize().unwrap();
            assert!(p.starts_with(&self.1) && p != self.1);
            fs::remove_dir_all(p).unwrap();
        }
    }
    fn spec() -> save::Spec {
        save::check(
            json!({"operation":"con","session":"active","dry":false,"confirm":"server-write"})
                .as_object()
                .unwrap(),
        )
        .unwrap()
    }
    fn read() -> Result<(String, usize), String> {
        Ok(("active".into(), 0))
    }

    #[test]
    fn preview_lost_reply_replay_and_reopen_never_duplicate_save_requests() {
        let f = Fixture::new();
        let mut state = State::default();
        let mut request = spec();
        request.dry = true;
        let dry = run_with(
            &f.0,
            &mut state,
            &request,
            "active",
            read,
            || panic!(),
            || Duration::ZERO,
        )
        .unwrap();
        assert_eq!(dry["state"], "dry");
        assert!(!f.0.join("server-save").exists());
        f.enable();
        request.dry = false;
        let result = run_with(
            &f.0,
            &mut state,
            &request,
            "active",
            read,
            || Ok(()),
            || Duration::ZERO,
        )
        .unwrap();
        assert_eq!(result["save_request_accepted"], true);
        assert_eq!(result["save_completed_verified"], false);
        assert_eq!(result["causal_completion_verified"], false);
        assert_eq!(f.records(), 2);
        assert!(f.0.join("server-save/save-con.json").is_file());
        // A lost result is reconciled by the same identity, never a second dispatch.
        assert_eq!(
            run_with(
                &f.0,
                &mut state,
                &request,
                "active",
                || panic!(),
                || panic!(),
                || Duration::ZERO
            )
            .unwrap(),
            result
        );
        assert!(
            run_with(
                &f.0,
                &mut State::default(),
                &request,
                "active",
                read,
                || panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
        request.dry = true;
        request.session = None;
        assert_eq!(
            run_with(
                &f.0,
                &mut state,
                &request,
                "active",
                || panic!(),
                || panic!(),
                || Duration::ZERO
            )
            .unwrap(),
            result
        );
        assert_eq!(
            run_with(
                &f.0,
                &mut State::default(),
                &request,
                "active",
                || panic!(),
                || panic!(),
                || Duration::ZERO
            )
            .unwrap()["state"],
            "durable_outcome_unknown"
        );
        assert!(
            run_with(
                &f.0,
                &mut state,
                &request,
                "other",
                read,
                || panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        assert_eq!(f.records(), 2);
    }
    #[test]
    fn changed_roster_session_flag_and_deadline_after_intent_never_dispatch() {
        for mode in ["roster", "session", "flag", "deadline"] {
            let f = Fixture::new();
            f.enable();
            let mut state = State::default();
            let reads = Cell::new(0);
            let time = Cell::new(Duration::ZERO);
            assert!(
                run_with(
                    &f.0,
                    &mut state,
                    &spec(),
                    "active",
                    || {
                        reads.set(reads.get() + 1);
                        if reads.get() == 2 {
                            match mode {
                                "roster" => return Ok(("active".into(), 1)),
                                "session" => return Ok(("other".into(), 0)),
                                "flag" => fs::remove_file(f.0.join(write_gate::FLAG)).unwrap(),
                                "deadline" => time.set(save::WAIT),
                                _ => unreachable!(),
                            }
                        }
                        read()
                    },
                    || panic!(),
                    || time.get()
                )
                .is_err()
            );
            assert_eq!(state.operations["con"].1["state"], "not_submitted");
            assert_eq!(f.records(), 2);
        }
    }
    #[test]
    fn missing_admission_intent_native_and_outcome_errors_never_retry() {
        for mode in ["flag", "context", "intent", "native", "outcome"] {
            let f = Fixture::new();
            let mut state = State::default();
            if mode != "flag" {
                f.enable();
            }
            if mode == "intent" {
                fs::create_dir(f.0.join("journal.jsonl")).unwrap();
            }
            let calls = Cell::new(0);
            let result = run_with(
                &f.0,
                &mut state,
                &spec(),
                "active",
                || {
                    if mode == "context" {
                        Err("unavailable".into())
                    } else {
                        read()
                    }
                },
                || {
                    calls.set(calls.get() + 1);
                    if mode == "outcome" {
                        fs::remove_file(f.0.join("journal.jsonl")).unwrap();
                        fs::create_dir(f.0.join("journal.jsonl")).unwrap();
                    }
                    if mode == "native" {
                        Err("native unavailable".into())
                    } else {
                        Ok(())
                    }
                },
                || Duration::ZERO,
            );
            if ["flag", "context", "intent"].contains(&mode) {
                assert!(result.is_err());
                assert_eq!(calls.get(), 0);
            } else {
                let receipt = result.unwrap();
                assert_eq!(calls.get(), 1);
                assert_eq!(receipt["save_completed_verified"], false);
                assert_eq!(receipt["save_request_accepted"], mode != "native");
                if mode == "outcome" {
                    assert!(receipt.get("journal_error").is_some());
                }
                assert_eq!(
                    run_with(
                        &f.0,
                        &mut state,
                        &spec(),
                        "active",
                        || panic!(),
                        || panic!(),
                        || Duration::ZERO
                    )
                    .unwrap(),
                    receipt
                );
            }
            if mode == "intent" {
                assert_eq!(
                    run_with(
                        &f.0,
                        &mut state,
                        &spec(),
                        "active",
                        || panic!(),
                        || panic!(),
                        || Duration::ZERO
                    )
                    .unwrap()["state"],
                    "not_issued_unknown"
                );
            }
        }
    }
    #[test]
    fn durable_namespace_capacity_and_two_record_headroom_refuse_before_dispatch() {
        let f = Fixture::new();
        f.enable();
        fs::create_dir(f.0.join("server-save")).unwrap();
        for i in 0..64 {
            fs::write(
                f.0.join("server-save").join(format!("save-{i}.json")),
                b"{}",
            )
            .unwrap();
        }
        assert!(
            run_with(
                &f.0,
                &mut State::default(),
                &spec(),
                "active",
                read,
                || panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        assert!(!f.0.join("server-save/save-con.json").exists());
        let g = Fixture::new();
        g.enable();
        let size = (policy::construction::JOURNAL_MAX_BYTES - 2 * write_gate::JOURNAL_RECORD_BYTES
            + 1) as u64;
        let file = fs::File::create(g.0.join("journal.jsonl")).unwrap();
        file.set_len(size).unwrap();
        drop(file);
        assert!(
            run_with(
                &g.0,
                &mut State::default(),
                &spec(),
                "active",
                read,
                || panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        assert_eq!(fs::metadata(g.0.join("journal.jsonl")).unwrap().len(), size);
        assert!(g.0.join("server-save/save-con.json").is_file());
    }
    static ADAPTER: Mutex<Option<(SaveControl, World, Scheduler)>> = Mutex::new(None);
    struct Probe;
    impl enshwick::Mod for Probe {
        fn init(load: &mut enshwick::Load<'_>) -> Result<Self, enshwick::Off> {
            let control = SaveControl::new(load.mod_dir()?, Construction::new(load)?);
            *ADAPTER.lock().unwrap() = Some((control, load.world()?, load.scheduler()?));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.save"
side = "server""#
    );
    #[test]
    fn scheduled_save_cancels_late_work_preserves_native_refusal_and_invalidates_stale_sessions() {
        let f = Fixture::new();
        f.enable();
        let loader = enshwick::test::FakeHost::new()
            .world()
            .mod_dir(f.0.to_str().unwrap())
            .install();
        // SAFETY: this module's Probe generates the init export.
        assert_eq!(
            unsafe { loader.init(enshwick_mod_init) },
            enshwick::abi::Status::OK
        );
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let (control, world, scheduler) = ADAPTER.lock().unwrap().as_ref().unwrap().clone();
        let drive = |request: save::Spec| {
            let adapter = control.clone();
            let task = std::thread::spawn(move || adapter.call(world, scheduler, request));
            let deadline = Instant::now() + Duration::from_secs(8);
            while !task.is_finished() {
                assert!(Instant::now() < deadline);
                let _ = loader.tick(Duration::from_millis(50));
                std::thread::sleep(Duration::from_millis(1));
            }
            task.join().unwrap()
        };
        let mut request = spec();
        request.dry = true;
        request.session = None;
        let preview = drive(request.clone()).unwrap();
        assert_eq!(loader.saves(), 0);
        request.dry = false;
        request.session = Some(preview["session"].as_str().unwrap().into());
        let adapter = control.clone();
        let late = request.clone();
        assert!(
            std::thread::spawn(move || adapter.call(world, scheduler, late))
                .join()
                .unwrap()
                .unwrap_err()
                .contains("queued request cancelled")
        );
        let _ = loader.tick(Duration::from_millis(50));
        assert_eq!(loader.saves(), 0);
        assert!(!f.0.join("server-save").exists());
        let receipt = drive(request.clone()).unwrap();
        assert_eq!(loader.saves(), 1);
        assert_eq!(receipt["save_request_accepted"], true);
        assert_eq!(drive(request.clone()).unwrap(), receipt);
        assert_eq!(loader.saves(), 1);
        world.hold_saves(true).unwrap();
        request.operation = "held".into();
        let refused = drive(request.clone()).unwrap();
        assert_eq!(refused["save_request_accepted"], false);
        assert_eq!(refused["state"], "save_error_outcome_unknown");
        assert_eq!(loader.saves(), 1);
        world.hold_saves(false).unwrap();
        assert_eq!(drive(request.clone()).unwrap(), refused);
        assert_eq!(loader.saves(), 1);
        request.operation = "deliberate-new-request".into();
        assert_eq!(
            drive(request.clone()).unwrap()["save_request_accepted"],
            true
        );
        assert_eq!(loader.saves(), 2);
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        assert!(drive(request).is_err());
        assert_eq!(loader.saves(), 2);
        assert_eq!(f.records(), 6);
        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
    }
}
