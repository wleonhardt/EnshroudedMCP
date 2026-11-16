//! Deliberate native shutdown requests; acceptance is not process/save proof.
use crate::{construction::Construction, tools::GameTools};
use enshwick::{schedule::Scheduler, world::World};
use policy::{
    server_shutdown as shutdown,
    write_gate::{self, PendingWrite},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, TryLockError, mpsc},
    time::{Duration, Instant},
};

#[derive(Default, Debug)]
struct State {
    operations: BTreeMap<String, (String, Value)>,
    attempted_session: Option<String>,
    pending_operation: Option<String>,
    lifecycle: shutdown::Lifecycle,
    event_sources: [bool; 2],
    generation_source: bool,
    observation_epoch: u64,
    attempted_observation_epoch: Option<u64>,
}
#[derive(Clone, Debug)]
pub(crate) struct Shutdown {
    dir: PathBuf,
    lifecycle: Construction,
    state: Arc<Mutex<State>>,
}
pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "server_shutdown",
        description: "ADMIN WRITE: request published native dedicated-server graceful shutdown/final save. Dry by default; supply an operation identity. Real request requires preview session, empty SDK player-character roster, fresh flag and confirm server-write. Roster is not a connection lock; coordinate access. Existing identities return retained status; never automatically retry. One native attempt per active session. Acceptance does not prove process exit or final save. No automatic restart, countdown, save holds, process kill or client logout.",
        schema: shutdown::schema(),
    }
}
pub(crate) fn status_tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "server_shutdown_status",
        description: "Read a retained current-boot shutdown record without an active world, write flag, scheduler or native call. Historical receipt/lifecycle observations only; no current-world, process-exit or final-save proof. Busy state is unavailable; query the same identity later, never submit another shutdown. Prior-boot identities remain unavailable without reconstruction. No retry, restart or journal append.",
        schema: shutdown::status_schema(),
    }
}
fn gate(spec: &shutdown::Spec, dir: &Path) -> Result<(), String> {
    write_gate::check(
        spec.dry,
        true,
        write_gate::flag_age(&dir.join(write_gate::FLAG)),
        spec.confirm.as_deref(),
    )
}
fn path(dir: &Path, operation: &str) -> PathBuf {
    dir.join("server-shutdown")
        .join(format!("stop-{operation}.json"))
}
fn capacity(dir: &Path, count: usize) -> Result<(), String> {
    shutdown::capacity(count)?;
    match fs::read_dir(dir.join("server-shutdown")) {
        Ok(entries) => {
            for (index, entry) in entries.enumerate() {
                entry.map_err(|e| e.to_string())?;
                shutdown::capacity(index + 1)?;
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
    fs::create_dir_all(dir.join("server-shutdown")).map_err(|e| e.to_string())?;
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
        "server_shutdown",
        &json!({"kind":"server_shutdown_outcome","operation":operation,"outcome":value}),
        0,
    ) {
        value["journal_error"] = json!(error);
    }
}
fn observe(dir: &Path, state: &mut State, generation: u64, event: shutdown::LifecycleEvent) {
    if !shutdown::observes(generation, state.attempted_observation_epoch) {
        return;
    }
    let Some(operation) = state.pending_operation.clone() else {
        return;
    };
    let Some(kind) = state.lifecycle.record(event) else {
        return;
    };
    let Some((session, value)) = state.operations.get_mut(&operation) else {
        return;
    };
    value["lifecycle_observations"] = json!({"shutdown_started":state.lifecycle.shutdown_started,"world_saved_after_shutdown":state.lifecycle.world_saved_after_shutdown,"source":"published_sdk_events","causal_completion_verified":false});
    if let Err(error) = GameTools::journal(
        Some(dir),
        "server_shutdown",
        &json!({"kind":"server_shutdown_lifecycle","operation":operation,"session":session,"event":kind,"source":"published_sdk_events","causal_completion_verified":false}),
        0,
    ) {
        value["lifecycle_journal_error"] = json!(error);
    }
}

impl Shutdown {
    pub(crate) fn status(&self, operation: &str) -> Result<Value, String> {
        let mut report = json!({"operation":operation,"source":"retained_shutdown_record",
            "state":"busy","native_outcome":null,"read_only":true,
            "current_world_context_verified":false,"native_identity_reconstructed":false,
            "automatic_retry":false,"automatic_restart":false,
            "process_exit_verified":false,"final_save_verified":false});
        let (retained, durable) = match self.state.try_lock() {
            Ok(state) => {
                let retained = state
                    .operations
                    .get(operation)
                    .map(|(_, value)| value.clone());
                let durable = retained.is_none() && path(&self.dir, operation).exists();
                (retained, durable)
            }
            Err(TryLockError::WouldBlock) => return Ok(report),
            Err(TryLockError::Poisoned(_)) => return Err("shutdown state poisoned".into()),
        };
        if let Some(value) = retained {
            report["state"] = json!("retained");
            report["native_outcome"] = value;
        } else if durable {
            report["state"] = json!("prior_boot_unavailable");
        } else {
            return Err("unknown shutdown operation".into());
        }
        Ok(report)
    }
    pub(crate) fn new(dir: PathBuf, lifecycle: Construction) -> Self {
        Self {
            dir,
            lifecycle,
            state: Arc::default(),
        }
    }
    pub(crate) fn register_events(&self, load: &mut enshwick::Load<'_>) {
        let started = self.clone();
        let shutdown = load.on::<enshwick::events::ShutdownStarted>(move |()| {
            started.record_event(shutdown::LifecycleEvent::ShutdownStarted)
        });
        if let Err(error) = &shutdown {
            enshwick::warn!("shutdown lifecycle event unavailable: {error}");
        }
        let saved = self.clone();
        let save = load.on::<enshwick::events::WorldSaved>(move |()| {
            saved.record_event(shutdown::LifecycleEvent::WorldSaved)
        });
        if let Err(error) = &save {
            enshwick::warn!("shutdown save event unavailable: {error}");
        }
        let loaded = self.clone();
        let generation = load.on::<enshwick::events::WorldLoaded>(move |()| {
            if let Ok(mut state) = loaded.state.lock() {
                state.observation_epoch = state.observation_epoch.saturating_add(1);
            }
        });
        if let Err(error) = &generation {
            enshwick::warn!("shutdown lifecycle generation unavailable: {error}");
        }
        if let Ok(mut state) = self.state.lock() {
            state.event_sources = [shutdown.is_ok(), save.is_ok()];
            state.generation_source = generation.is_ok();
        }
    }
    fn record_event(&self, event: shutdown::LifecycleEvent) {
        // This generation stays stable through stop; WorldLoaded advances it.
        if let Ok(mut state) = self.state.lock() {
            let enabled = match event {
                shutdown::LifecycleEvent::ShutdownStarted => state.event_sources[0],
                shutdown::LifecycleEvent::WorldSaved => state.event_sources[1],
            };
            if state.generation_source && enabled {
                let epoch = state.observation_epoch;
                observe(&self.dir, &mut state, epoch, event);
            }
        }
    }
    pub(crate) fn call(
        &self,
        world: World,
        scheduler: Scheduler,
        spec: shutdown::Spec,
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
                let mut state = this.state.lock().map_err(|_| "shutdown state poisoned")?;
                run_with(
                    &this.dir,
                    &mut state,
                    &spec,
                    &active,
                    || Ok((this.lifecycle.active_session()?, world.players().len())),
                    || world.stop().map_err(|e| e.to_string()),
                    || started.elapsed(),
                )
            })();
            let _ = tx.send(result);
        });
        rx.recv_timeout(shutdown::WAIT).map_err(|_|if pending.cancel(){"shutdown timed out before execution; queued request cancelled".to_owned()}else{"shutdown started; outcome unknown; inspect journal and exact process, never blindly retry".to_owned()})?
    }
}
fn run_with(
    dir: &Path,
    state: &mut State,
    spec: &shutdown::Spec,
    session: &str,
    mut read: impl FnMut() -> Result<(String, usize), String>,
    stop: impl FnOnce() -> Result<(), String>,
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
                json!({"operation":spec.operation,"state":"durable_outcome_unknown","automatic_retry":false,"automatic_restart":false,"process_exit_verified":false,"final_save_verified":false}),
            )
        } else {
            Err("durable shutdown identity exists; never redispatch".into())
        };
    }
    shutdown::elapsed(elapsed())?;
    capacity(dir, state.operations.len())?;
    shutdown::attempt(session, state.attempted_session.as_deref())?;
    let (current, players) = read()?;
    policy::construction::check_session(session, &current)?;
    shutdown::admit(spec, &current, players)?;
    let mut value = json!({"operation":spec.operation,"session":session,"state":"dry","lifecycle_event_sources":{"shutdown_started":state.event_sources[0],"world_saved":state.event_sources[1],"world_generation":state.generation_source},"lifecycle_observations":{"shutdown_started":false,"world_saved_after_shutdown":false,"source":"published_sdk_events","causal_completion_verified":false},"observed_player_characters":players,"stop_request_accepted":false,"stop_dispatched":false,"process_exit_verified":false,"final_save_verified":false,"automatic_retry":false,"automatic_restart":false,"native_undo":false,"scope":"dedicated-server request; empty character roster is not a connection lock; native availability and final save are not proven by preview"});
    if spec.dry {
        return Ok(value);
    }
    let intent =
        json!({"kind":"server_shutdown_intent","operation":spec.operation,"session":session});
    value["state"] = json!("not_issued_unknown");
    state
        .operations
        .insert(spec.operation.clone(), (session.into(), value));
    if let Err(error) = reserve(dir, &spec.operation, &intent).and_then(|()| {
        let size = match fs::metadata(dir.join("journal.jsonl")) {
            Ok(meta) if meta.is_file() => meta.len(),
            Ok(_) => return Err("shutdown journal must be a regular file".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(error.to_string()),
        };
        shutdown::journal_headroom(size)?;
        GameTools::journal(
            Some(dir),
            "server_shutdown",
            &intent,
            write_gate::JOURNAL_RECORD_BYTES,
        )
    }) {
        state.operations.get_mut(&spec.operation).unwrap().1["journal_error"] = json!(error);
        return Err("shutdown intent failed; identity blocked, no native request issued".into());
    }
    let ready = (|| {
        shutdown::elapsed(elapsed())?;
        let (current, players) = read()?;
        policy::construction::check_session(session, &current)?;
        shutdown::admit(spec, &current, players)?;
        gate(spec, dir)?;
        shutdown::elapsed(elapsed())
    })();
    let value = &mut state.operations.get_mut(&spec.operation).unwrap().1;
    if let Err(error) = ready {
        value["state"] = json!("not_submitted");
        value["refusal"] = json!(error);
        outcome(dir, &spec.operation, value);
        return Err(
            "shutdown admission changed; identity blocked, no native request issued".into(),
        );
    }
    state.attempted_session = Some(session.into());
    state.pending_operation = Some(spec.operation.clone());
    state.attempted_observation_epoch = Some(state.observation_epoch);
    state.lifecycle = shutdown::Lifecycle::default();
    value["state"] = json!("issued_unknown");
    value["stop_dispatched"] = json!(true);
    match stop() {
        Ok(()) => {
            value["state"] = json!("stop_accepted_exit_unverified");
            value["stop_request_accepted"] = json!(true);
        }
        Err(error) => {
            value["state"] = json!("stop_error_outcome_unknown");
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
                "shutdown-{}-{}",
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
    fn spec() -> shutdown::Spec {
        shutdown::check(
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
    fn preview_replay_and_reopen_never_issue_extra_shutdowns() {
        let f = Fixture::new();
        let mut state = State::default();
        let mut request = spec();
        request.dry = true;
        assert_eq!(
            run_with(
                &f.0,
                &mut state,
                &request,
                "active",
                read,
                || panic!(),
                || Duration::ZERO
            )
            .unwrap()["state"],
            "dry"
        );
        assert!(!f.0.join("server-shutdown").exists());
        f.enable();
        request.dry = false;
        let calls = Cell::new(0);
        let result = run_with(
            &f.0,
            &mut state,
            &request,
            "active",
            read,
            || {
                calls.set(calls.get() + 1);
                Ok(())
            },
            || Duration::ZERO,
        )
        .unwrap();
        assert_eq!(result["stop_request_accepted"], true);
        assert_eq!(result["process_exit_verified"], false);
        assert_eq!(result["final_save_verified"], false);
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
        assert!(f.0.join("server-shutdown/stop-con.json").is_file());
        assert_eq!(calls.get(), 1);
        assert_eq!(f.records(), 2);
        request.operation = "another".into();
        assert!(
            run_with(
                &f.0,
                &mut state,
                &request,
                "active",
                read,
                || panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        request.operation = "con".into();
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
    }
    #[test]
    fn changed_gates_after_intent_never_dispatch() {
        for mode in ["roster", "session", "flag", "deadline"] {
            let f = Fixture::new();
            f.enable();
            let mut state = State::default();
            let reads = Cell::new(0);
            let time = Cell::new(Duration::ZERO);
            let result = run_with(
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
                            "deadline" => time.set(shutdown::WAIT),
                            _ => unreachable!(),
                        }
                    }
                    read()
                },
                || panic!(),
                || time.get(),
            );
            assert!(result.is_err());
            assert_eq!(state.operations["con"].1["state"], "not_submitted");
            assert_eq!(f.records(), 2);
        }
    }
    #[test]
    fn intent_failures_native_errors_and_outcome_failures_never_retry() {
        for mode in [
            "missing_gate",
            "missing_context",
            "intent",
            "native",
            "outcome",
        ] {
            let f = Fixture::new();
            let mut state = State::default();
            if mode != "missing_gate" {
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
                    if mode == "missing_context" {
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
            if ["missing_gate", "missing_context", "intent"].contains(&mode) {
                assert!(result.is_err());
                assert_eq!(calls.get(), 0);
            } else {
                let receipt = result.unwrap();
                if mode == "native" {
                    assert_eq!(receipt["state"], "stop_error_outcome_unknown");
                } else {
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
                let mut another = spec();
                another.operation = "next".into();
                assert!(
                    run_with(
                        &f.0,
                        &mut state,
                        &another,
                        "active",
                        read,
                        || panic!(),
                        || Duration::ZERO
                    )
                    .is_err()
                );
                assert_eq!(calls.get(), 1);
            }
        }
    }
    #[test]
    fn full_durable_namespace_refuses_new_requests_and_reports_old_unknown() {
        let f = Fixture::new();
        fs::create_dir(f.0.join("server-shutdown")).unwrap();
        for i in 0..shutdown::MAX_OPERATIONS {
            fs::write(
                f.0.join("server-shutdown")
                    .join(format!("stop-old-{i}.json")),
                b"{}",
            )
            .unwrap();
        }
        let mut request = spec();
        request.dry = true;
        let mut state = State::default();
        assert!(
            run_with(
                &f.0,
                &mut state,
                &request,
                "active",
                read,
                || panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        request.operation = "old-0".into();
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
            .unwrap()["state"],
            "durable_outcome_unknown"
        );
        assert!(!f.0.join("journal.jsonl").exists());
    }

    #[test]
    fn lifecycle_audit_is_ordered_session_bound_and_never_promotes_receipt() {
        let f = Fixture::new();
        f.enable();
        let mut state = State::default();
        observe(
            &f.0,
            &mut state,
            0,
            shutdown::LifecycleEvent::ShutdownStarted,
        );
        assert!(!f.0.join("journal.jsonl").exists());
        let receipt = run_with(
            &f.0,
            &mut state,
            &spec(),
            "active",
            read,
            || Ok(()),
            || Duration::ZERO,
        )
        .unwrap();
        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
        observe(
            &f.0,
            &mut state,
            1,
            shutdown::LifecycleEvent::ShutdownStarted,
        );
        observe(&f.0, &mut state, 0, shutdown::LifecycleEvent::WorldSaved);
        assert_eq!(f.records(), 2);
        observe(
            &f.0,
            &mut state,
            0,
            shutdown::LifecycleEvent::ShutdownStarted,
        );
        observe(
            &f.0,
            &mut state,
            0,
            shutdown::LifecycleEvent::ShutdownStarted,
        );
        observe(&f.0, &mut state, 0, shutdown::LifecycleEvent::WorldSaved);
        observe(&f.0, &mut state, 0, shutdown::LifecycleEvent::WorldSaved);
        assert_eq!(f.records(), 4);
        let value = &state.operations["con"].1;
        assert_eq!(value["state"], receipt["state"]);
        assert_eq!(value["process_exit_verified"], false);
        assert_eq!(value["final_save_verified"], false);
        assert_eq!(value["lifecycle_observations"]["shutdown_started"], true);
        assert_eq!(
            value["lifecycle_observations"]["world_saved_after_shutdown"],
            true
        );
    }
    #[test]
    fn lifecycle_journal_failure_stays_explicit_and_does_not_repeat() {
        let f = Fixture::new();
        f.enable();
        let mut state = State::default();
        run_with(
            &f.0,
            &mut state,
            &spec(),
            "active",
            read,
            || Ok(()),
            || Duration::ZERO,
        )
        .unwrap();
        fs::remove_file(f.0.join("journal.jsonl")).unwrap();
        fs::create_dir(f.0.join("journal.jsonl")).unwrap();
        observe(
            &f.0,
            &mut state,
            0,
            shutdown::LifecycleEvent::ShutdownStarted,
        );
        let result = state.operations["con"].1.clone();
        assert!(result.get("lifecycle_journal_error").is_some());
        observe(
            &f.0,
            &mut state,
            0,
            shutdown::LifecycleEvent::ShutdownStarted,
        );
        assert_eq!(state.operations["con"].1, result);
    }

    #[test]
    fn audit_headroom_is_required_before_native_dispatch() {
        let f = Fixture::new();
        f.enable();
        let size =
            (policy::construction::JOURNAL_MAX_BYTES - 2 * write_gate::JOURNAL_RECORD_BYTES) as u64;
        let file = fs::File::create(f.0.join("journal.jsonl")).unwrap();
        file.set_len(size).unwrap();
        drop(file);
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
        assert_eq!(fs::metadata(f.0.join("journal.jsonl")).unwrap().len(), size);
        assert!(f.0.join("server-shutdown/stop-con.json").is_file());
    }
    static ADAPTER: Mutex<Option<(Shutdown, World, Scheduler)>> = Mutex::new(None);
    struct Probe;
    impl enshwick::Mod for Probe {
        fn init(load: &mut enshwick::Load<'_>) -> Result<Self, enshwick::Off> {
            let lifecycle = Construction::new(load)?;
            let control = Shutdown::new(load.mod_dir()?, lifecycle);
            control.register_events(load);
            *ADAPTER.lock().unwrap() = Some((control, load.world()?, load.scheduler()?));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.shutdown"
side = "server""#
    );
    #[test]
    fn scheduled_native_stop_cancels_late_work_and_does_not_fabricate_exit() {
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
        assert!(
            control
                .status("absent")
                .unwrap_err()
                .contains("unknown shutdown")
        );
        {
            let _held = control.state.lock().unwrap();
            let busy = control.status("con").unwrap();
            assert_eq!(busy["state"], "busy");
            assert!(busy["native_outcome"].is_null());
        }
        let drive = |request: shutdown::Spec| {
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
        assert_eq!(
            preview["lifecycle_event_sources"],
            json!({"shutdown_started":true,"world_saved":true,"world_generation":true})
        );
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
        assert!(!loader.stop_asked());
        assert!(!f.0.join("server-shutdown").exists());
        let receipt = match drive(request.clone()) {
            Ok(receipt) => receipt,
            Err(error) => {
                assert_eq!(
                    error,
                    "shutdown started; outcome unknown; inspect journal and exact process, never blindly retry"
                );
                let reconciled = control.status("con").unwrap();
                assert_eq!(reconciled["state"], "retained");
                reconciled["native_outcome"].clone()
            }
        };
        assert!(loader.stop_asked());
        assert_eq!(receipt["stop_request_accepted"], true);
        assert_eq!(receipt["process_exit_verified"], false);
        assert_eq!(receipt["final_save_verified"], false);
        assert_eq!(drive(request).unwrap(), receipt);
        assert_eq!(f.records(), 2);
        fs::remove_file(f.0.join(write_gate::FLAG)).unwrap();
        let retained = control.status("con").unwrap();
        assert_eq!(retained["native_outcome"], receipt);
        let listener = control.clone();
        assert_eq!(
            std::thread::spawn(move || listener.status("con"))
                .join()
                .unwrap()
                .unwrap(),
            retained
        );
        for field in [
            "automatic_retry",
            "automatic_restart",
            "process_exit_verified",
            "final_save_verified",
            "current_world_context_verified",
            "native_identity_reconstructed",
        ] {
            assert_eq!(retained[field], false);
        }
        assert_eq!(f.records(), 2);
        assert!(loader.raise::<enshwick::events::WorldSaved>(&()) > 0);
        assert_eq!(f.records(), 2);
        assert!(loader.raise::<enshwick::events::ShutdownStarted>(&()) > 0);
        assert_eq!(f.records(), 3);
        let stopped = control.status("con").unwrap();
        assert_eq!(
            stopped["native_outcome"]["lifecycle_observations"]["shutdown_started"],
            true
        );
        assert_eq!(stopped["process_exit_verified"], false);
        assert!(loader.raise::<enshwick::events::WorldSaved>(&()) > 0);
        assert_eq!(f.records(), 4);
        assert!(loader.raise::<enshwick::events::WorldSaved>(&()) > 0);
        assert_eq!(f.records(), 4);
        let state = control.state.lock().unwrap();
        assert_eq!(
            state.operations["con"].1["lifecycle_observations"]["world_saved_after_shutdown"],
            true
        );
        assert_eq!(state.operations["con"].1["final_save_verified"], false);
        drop(state);
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        assert!(loader.raise::<enshwick::events::WorldSaved>(&()) > 0);
        assert_eq!(f.records(), 4);
        let status = shutdown::check(json!({"operation":"con"}).as_object().unwrap()).unwrap();
        assert!(drive(status).is_err());
        let historical = control.status("con").unwrap();
        assert_eq!(historical["state"], "retained");
        assert_eq!(historical["current_world_context_verified"], false);
        let audit = fs::read(f.0.join("journal.jsonl")).unwrap();
        control.state.lock().unwrap().operations.clear();
        fs::write(path(&f.0, "con"), b"not a decodable reservation").unwrap();
        let prior = control.status("con").unwrap();
        assert_eq!(prior["state"], "prior_boot_unavailable");
        assert!(prior["native_outcome"].is_null());
        assert_eq!(fs::read(f.0.join("journal.jsonl")).unwrap(), audit);
        assert!(loader.stop_asked());
    }
}
