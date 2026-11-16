//! Deliberate forward clock requests with durable once-only identities.

use crate::{
    construction::Construction,
    tools::{Game, GameTools},
};
use policy::{
    clock_control as clock,
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

#[derive(Clone, Debug)]
pub(crate) struct ClockControl {
    dir: PathBuf,
    construction: Construction,
    operations: Arc<Mutex<BTreeMap<String, Operation>>>,
}

#[derive(Debug)]
struct Operation {
    request: Value,
    session: String,
    outcome: Value,
}

struct QueuedClock {
    pending: Arc<PendingWrite>,
    reply: mpsc::Receiver<Result<Value, String>>,
}

impl QueuedClock {
    fn wait(self, timeout: Duration) -> Result<Value, String> {
        self.reply.recv_timeout(timeout).map_err(|_| {
            if self.pending.cancel() { "clock request timed out before execution; queued request cancelled".to_owned() }
            else { "clock request started; outcome unknown; inspect operation status and journal, never blindly retry".to_owned() }
        })?
    }
}

pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "clock_advance",
        description: "ADMIN WRITE: request the next occurrence of an integer minute of day. Dry by default, one to twelve hours forward from the latest observation, paused/normal speed and empty SDK player-character roster required. Roster is not a connection lock. Real calls require session, expected_hours, fresh flag and confirmation. Durable operation replay never redispatches. Operation alone reads retained status. Advances world progression; not undo, a lighting override or a causal completion receipt. No automatic retry or speed changes.",
        schema: clock::schema(),
    }
}

fn gate(spec: &clock::Spec, dir: &Path) -> Result<(), String> {
    write_gate::check(
        spec.dry || spec.target_minute.is_none(),
        true,
        write_gate::flag_age(&dir.join(write_gate::FLAG)),
        spec.confirm.as_deref(),
    )
}

fn operation_path(dir: &Path, operation: &str) -> PathBuf {
    // The prefix also keeps otherwise valid IDs away from Windows device names.
    dir.join("clock-operations")
        .join(format!("clock-{operation}.json"))
}

fn reserve(dir: &Path, operation: &str, intent: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(intent).map_err(|e| e.to_string())?;
    clock::check_record_size(bytes.len())?;
    let folder = dir.join("clock-operations");
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(operation_path(dir, operation))
        .map_err(|e| format!("clock identity reserved or unavailable; never redispatch: {e}"))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("clock intent durability failed: {e}"))
}

fn capacity(dir: &Path, operations: usize) -> Result<(), String> {
    clock::check_capacity(operations)?;
    match fs::read_dir(dir.join("clock-operations")) {
        Ok(entries) => {
            for (index, entry) in entries.enumerate() {
                entry.map_err(|e| e.to_string())?;
                clock::check_capacity(index + 1)?;
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

impl ClockControl {
    pub(crate) fn new(dir: PathBuf, construction: Construction) -> Self {
        Self {
            dir,
            construction,
            operations: Arc::default(),
        }
    }

    fn context(&self, game: Game) -> Result<clock::Context, String> {
        let session = self.construction.active_session()?;
        let now = game.clock.now().ok_or("native server clock unavailable")?;
        Ok(clock::Context {
            session,
            hours: f64::from(now.hours),
            speed: f64::from(now.speed),
            players: game.world.players().len(),
        })
    }

    pub(crate) fn call(&self, game: Game, spec: clock::Spec) -> Result<Value, String> {
        self.queue(game, spec, || {
            let started = Instant::now();
            move || started.elapsed()
        })?
        .wait(clock::WAIT)
    }

    fn queue<F: Fn() -> Duration + Send + 'static>(
        &self,
        game: Game,
        spec: clock::Spec,
        start_elapsed: impl FnOnce() -> F,
    ) -> Result<QueuedClock, String> {
        gate(&spec, &self.dir)?;
        let admitted = self.construction.active_session()?;
        let elapsed = start_elapsed();
        let pending = Arc::new(PendingWrite::default());
        let queued = pending.clone();
        let this = self.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        game.scheduler.next_tick(move || {
            if !queued.begin() {
                return;
            }
            let result = (|| {
                let active = this.construction.active_session()?;
                policy::construction::check_session(&admitted, &active)?;
                let mut operations = this
                    .operations
                    .lock()
                    .map_err(|_| "clock operation state poisoned")?;
                run_with(
                    &this.dir,
                    &mut operations,
                    &spec,
                    &active,
                    || this.context(game),
                    |target| game.clock.set_hours(target).map_err(|e| e.to_string()),
                    elapsed,
                )
            })();
            let _ = tx.send(result);
        });
        Ok(QueuedClock { pending, reply: rx })
    }
}

fn record_outcome(dir: &Path, operation: &str, op: &mut Operation) {
    if let Err(error) = GameTools::journal(
        Some(dir),
        "clock_advance",
        &json!({"kind":"clock_advance_outcome","operation":operation,"outcome":op.outcome}),
        0,
    ) {
        op.outcome["journal_error"] = json!(error);
    }
}

fn run_with(
    dir: &Path,
    operations: &mut BTreeMap<String, Operation>,
    spec: &clock::Spec,
    session: &str,
    mut read: impl FnMut() -> Result<clock::Context, String>,
    mut change: impl FnMut(f32) -> Result<(), String>,
    elapsed: impl Fn() -> Duration,
) -> Result<Value, String> {
    gate(spec, dir)?;
    if let Some(op) = operations.get(&spec.operation) {
        policy::construction::check_session(&op.session, session)?;
        if spec.target_minute.is_some() {
            clock::check_replay(&clock::canonical_request(spec), &op.request)?;
            return Ok(op.outcome.clone());
        }
        let mut result = op.outcome.clone();
        result["clock_now"] = json!(
            read()
                .ok()
                .map(|now| json!({"hours":now.hours,"speed":now.speed}))
        );
        return Ok(result);
    }
    if operation_path(dir, &spec.operation).exists() {
        if spec.target_minute.is_some() {
            return Err(
                "durable clock identity exists; outcome unknown or stale, never redispatch".into(),
            );
        }
        return Ok(
            json!({"operation":spec.operation,"state":"durable_outcome_unknown","automatic_retry":false,"redispatch":false,"native_undo":false}),
        );
    }
    if spec.target_minute.is_none() {
        return Err("unknown clock operation".into());
    }
    clock::check_elapsed(elapsed())?;
    capacity(dir, operations.len())?;
    let before = read()?;
    policy::construction::check_session(session, &before.session)?;
    let preview = clock::admit(spec, &before)?;
    let mut result = json!({"operation":spec.operation,"state":"dry","session":session,
        "expected_hours":before.hours,"target_minute":spec.target_minute,"target_hours":preview.target_hours,
        "forward_hours_from_observation":preview.forward_hours,"observed_speed":before.speed,
        "observed_player_characters":before.players,"world_mutated":false,"native_request_accepted":false,
        "automatic_retry":false,"native_undo":false,"elapsed_world_time_verified":false,"rendered_lighting_verified":false,
        "scope":"server forward progression; admission observations are not an atomic native elapsed-time bound; not a connection lock"});
    if spec.dry {
        return Ok(result);
    }
    let request = clock::canonical_request(spec);
    let intent = json!({"kind":"clock_advance_intent","operation":spec.operation,"request":request,
        "session":session,"before_hours":before.hours,"target_hours":preview.target_hours,"forward_hours_from_observation":preview.forward_hours,"native_undo":false});
    let mut op = Operation {
        request,
        session: session.into(),
        outcome: json!({"operation":spec.operation,"state":"not_issued_unknown","native_request_accepted":false,"world_mutated":false,"automatic_retry":false,"native_undo":false}),
    };
    if let Err(error) = reserve(dir, &spec.operation, &intent).and_then(|()| {
        GameTools::journal(
            Some(dir),
            "clock_advance",
            &intent,
            write_gate::JOURNAL_RECORD_BYTES,
        )
    }) {
        op.outcome["journal_error"] = json!(error);
        operations.insert(spec.operation.clone(), op);
        return Err("clock intent failed; identity blocked, no native request issued".into());
    }
    operations.insert(spec.operation.clone(), op);
    let ready = (|| {
        clock::check_elapsed(elapsed())?;
        let current = read()?;
        policy::construction::check_session(session, &current.session)?;
        let preview = clock::admit(spec, &current)?;
        gate(spec, dir)?;
        clock::check_elapsed(elapsed())?;
        Ok::<_, String>((current, preview))
    })();
    let op = operations
        .get_mut(&spec.operation)
        .ok_or("clock marker disappeared")?;
    let (current, preview) = match ready {
        Ok(value) => value,
        Err(error) => {
            op.outcome["state"] = json!("not_submitted");
            op.outcome["refusal"] = json!(error);
            record_outcome(dir, &spec.operation, op);
            return Err(
                "clock pre-dispatch admission changed; identity blocked, no native request issued"
                    .into(),
            );
        }
    };
    result["state"] = json!("issued_unknown");
    result["world_mutated"] = Value::Null;
    result["pre_dispatch_hours"] = json!(current.hours);
    result["forward_hours_from_observation"] = json!(preview.forward_hours);
    op.outcome = result; // The retained marker precedes the native attempt.
    let native = change(preview.target_hours);
    let after = read().ok().filter(|now| now.session == session);
    let near = clock::readback_near(preview.target_hours, after.as_ref());
    op.outcome["native_request_accepted"] = json!(native.is_ok());
    op.outcome["readback_near_target"] = json!(near);
    op.outcome["observed_after_hours"] = json!(after.map(|now| now.hours));
    op.outcome["state"] = json!(if native.is_err() {
        "command_error_unknown"
    } else if near {
        "clock_readback_near_target"
    } else {
        "readback_unverified"
    });
    if let Err(error) = native {
        op.outcome["native_error"] = json!(error);
    }
    record_outcome(dir, &spec.operation, op);
    Ok(op.outcome.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use policy::clock_control::{Context, Spec};
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::{cell::Cell, collections::BTreeMap, fs, path::PathBuf, time::Duration};

    struct Fixture(PathBuf, PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let base =
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../gamedata/tests");
            fs::create_dir_all(&base).unwrap();
            let base = base.canonicalize().unwrap();
            let path = base.join(format!(
                "clock-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path, base)
        }
        fn enable(&self) {
            fs::write(self.0.join(policy::write_gate::FLAG), []).unwrap();
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
            let resolved = self.0.canonicalize().unwrap();
            assert!(resolved.starts_with(&self.1) && resolved != self.1);
            fs::remove_dir_all(resolved).unwrap();
        }
    }
    fn context(hours: f64) -> Context {
        Context {
            session: "active".into(),
            hours,
            speed: 1.0,
            players: 0,
        }
    }
    fn spec() -> Spec {
        policy::clock_control::check(json!({"operation":"fixture","target_minute":720,"session":"active","expected_hours":8,"dry":false,"confirm":"server-write"}).as_object().unwrap()).unwrap()
    }

    #[test]
    fn dry_and_status_never_create_records_or_dispatch() {
        let f = Fixture::new();
        let mut ops = BTreeMap::new();
        let mut request = spec();
        request.dry = true;
        let value = run_with(
            &f.0,
            &mut ops,
            &request,
            "active",
            || Ok(context(8.0)),
            |_| panic!(),
            || Duration::ZERO,
        )
        .unwrap();
        assert_eq!(value["state"], "dry");
        assert_eq!(value["native_undo"], false);
        assert_eq!(value["world_mutated"], false);
        assert!(ops.is_empty());
        assert!(!f.0.join("clock-operations").exists() && !f.0.join("journal.jsonl").exists());
        let status =
            policy::clock_control::check(json!({"operation":"fixture"}).as_object().unwrap())
                .unwrap();
        assert!(
            run_with(
                &f.0,
                &mut ops,
                &status,
                "active",
                || panic!(),
                |_| panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
    }

    #[test]
    fn one_dispatch_survives_replay_alteration_and_a_reopened_controller() {
        let f = Fixture::new();
        f.enable();
        let mut ops = BTreeMap::new();
        let request = spec();
        let hours = Cell::new(8.0);
        let calls = Cell::new(0);
        let mut issue = |target| {
            calls.set(calls.get() + 1);
            hours.set(f64::from(target));
            Ok(())
        };
        let value = run_with(
            &f.0,
            &mut ops,
            &request,
            "active",
            || Ok(context(hours.get())),
            &mut issue,
            || Duration::ZERO,
        )
        .unwrap();
        assert_eq!(value["native_request_accepted"], true);
        assert_eq!(value["readback_near_target"], true);
        assert_eq!(calls.get(), 1);
        assert_eq!(f.records(), 2);
        assert!(f.0.join("clock-operations/clock-fixture.json").is_file());
        hours.set(13.0);
        let replay = run_with(
            &f.0,
            &mut ops,
            &request,
            "active",
            || Ok(context(hours.get())),
            &mut issue,
            || Duration::ZERO,
        )
        .unwrap();
        assert_eq!(replay, value);
        assert_eq!(hours.get(), 13.0);
        assert_eq!(calls.get(), 1);
        assert_eq!(f.records(), 2);
        let mut changed = request.clone();
        changed.target_minute = Some(780);
        assert!(
            run_with(
                &f.0,
                &mut ops,
                &changed,
                "active",
                || Ok(context(8.0)),
                &mut issue,
                || Duration::ZERO
            )
            .is_err()
        );
        hours.set(8.0);
        assert!(
            run_with(
                &f.0,
                &mut BTreeMap::new(),
                &request,
                "active",
                || Ok(context(hours.get())),
                &mut issue,
                || Duration::ZERO
            )
            .unwrap_err()
            .contains("durable")
        );
        fs::remove_file(f.0.join(policy::write_gate::FLAG)).unwrap();
        let status =
            policy::clock_control::check(json!({"operation":"fixture"}).as_object().unwrap())
                .unwrap();
        assert_eq!(
            run_with(
                &f.0,
                &mut ops,
                &status,
                "active",
                || Ok(context(hours.get())),
                &mut issue,
                || Duration::ZERO
            )
            .unwrap()["native_request_accepted"],
            true
        );
        assert!(
            run_with(
                &f.0,
                &mut ops,
                &status,
                "other",
                || Ok(context(hours.get())),
                &mut issue,
                || Duration::ZERO
            )
            .is_err()
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(f.records(), 2);
    }

    #[test]
    fn missing_gates_or_clock_and_durable_journal_failure_prevent_dispatch() {
        let f = Fixture::new();
        let mut ops = BTreeMap::new();
        let request = spec();
        assert!(
            run_with(
                &f.0,
                &mut ops,
                &request,
                "active",
                || Ok(context(8.0)),
                |_| panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        f.enable();
        assert!(
            run_with(
                &f.0,
                &mut ops,
                &request,
                "active",
                || Err("clock unavailable".into()),
                |_| panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        assert!(!f.0.join("clock-operations").exists());
        fs::create_dir(f.0.join("journal.jsonl")).unwrap();
        assert!(
            run_with(
                &f.0,
                &mut ops,
                &request,
                "active",
                || Ok(context(8.0)),
                |_| panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        assert!(f.0.join("clock-operations/clock-fixture.json").is_file());
        assert_eq!(ops["fixture"].outcome["state"], "not_issued_unknown");
    }

    #[test]
    fn changed_admission_after_intent_blocks_the_identity_without_dispatch() {
        for mode in ["flag", "deadline", "session", "roster"] {
            let f = Fixture::new();
            f.enable();
            let mut ops = BTreeMap::new();
            let request = spec();
            let reads = Cell::new(0);
            let elapsed = Cell::new(Duration::ZERO);
            let read = || {
                reads.set(reads.get() + 1);
                let mut current = context(8.0);
                if reads.get() == 2 {
                    match mode {
                        "flag" => fs::remove_file(f.0.join(policy::write_gate::FLAG)).unwrap(),
                        "deadline" => elapsed.set(Duration::from_secs(5)),
                        "session" => current.session = "other".into(),
                        "roster" => current.players = 1,
                        _ => unreachable!(),
                    }
                }
                Ok(current)
            };
            assert!(
                run_with(
                    &f.0,
                    &mut ops,
                    &request,
                    "active",
                    read,
                    |_| panic!(),
                    || elapsed.get()
                )
                .is_err()
            );
            assert_eq!(ops["fixture"].outcome["state"], "not_submitted");
            assert_eq!(f.records(), 2);
        }
    }

    #[test]
    fn native_errors_and_outcome_journal_failure_never_permit_replay() {
        for journal_failure in [false, true] {
            let f = Fixture::new();
            f.enable();
            let mut ops = BTreeMap::new();
            let request = spec();
            let calls = Cell::new(0);
            let hours = Cell::new(8.0);
            let change = |target| {
                calls.set(calls.get() + 1);
                if journal_failure {
                    hours.set(f64::from(target));
                    fs::remove_file(f.0.join("journal.jsonl")).unwrap();
                    fs::create_dir(f.0.join("journal.jsonl")).unwrap();
                    Ok(())
                } else {
                    Err("fixture native error".into())
                }
            };
            let value = run_with(
                &f.0,
                &mut ops,
                &request,
                "active",
                || Ok(context(hours.get())),
                change,
                || Duration::ZERO,
            )
            .unwrap();
            if journal_failure {
                assert!(value.get("journal_error").is_some());
            } else {
                assert_eq!(value["state"], "command_error_unknown");
            }
            assert_eq!(
                run_with(
                    &f.0,
                    &mut ops,
                    &request,
                    "active",
                    || panic!(),
                    |_| panic!(),
                    || Duration::ZERO
                )
                .unwrap(),
                value
            );
            assert_eq!(calls.get(), 1);
        }
    }

    #[test]
    fn windows_device_ids_are_prefixed_and_full_disk_namespace_refuses_new_requests() {
        let f = Fixture::new();
        f.enable();
        let mut ops = BTreeMap::new();
        let mut request = spec();
        request.operation = "con".into();
        let value = run_with(
            &f.0,
            &mut ops,
            &request,
            "active",
            || Ok(context(8.0)),
            |_| Ok(()),
            || Duration::ZERO,
        )
        .unwrap();
        assert_eq!(value["native_request_accepted"], true);
        assert!(f.0.join("clock-operations/clock-con.json").is_file());
        for index in 1..policy::clock_control::MAX_OPERATIONS {
            fs::write(
                f.0.join("clock-operations")
                    .join(format!("clock-extra-{index}.json")),
                b"{}",
            )
            .unwrap();
        }
        request.operation = "next".into();
        request.dry = true;
        assert!(
            run_with(
                &f.0,
                &mut ops,
                &request,
                "active",
                || Ok(context(8.0)),
                |_| panic!(),
                || Duration::ZERO
            )
            .is_err()
        );
        assert_eq!(f.records(), 2);
    }

    #[test]
    fn unavailable_mismatched_and_new_session_readbacks_remain_unverified() {
        for mode in ["unavailable", "mismatch", "session"] {
            let f = Fixture::new();
            f.enable();
            let mut ops = BTreeMap::new();
            let reads = Cell::new(0);
            let value = run_with(
                &f.0,
                &mut ops,
                &spec(),
                "active",
                || {
                    reads.set(reads.get() + 1);
                    if reads.get() < 3 {
                        return Ok(context(8.0));
                    }
                    match mode {
                        "unavailable" => Err("clock unavailable".into()),
                        "mismatch" => Ok(context(9.0)),
                        "session" => Ok(Context {
                            session: "other".into(),
                            ..context(12.0)
                        }),
                        _ => unreachable!(),
                    }
                },
                |_| Ok(()),
                || Duration::ZERO,
            )
            .unwrap();
            assert_eq!(value["state"], "readback_unverified");
            assert_eq!(value["native_request_accepted"], true);
            assert_eq!(value["world_mutated"], Value::Null);
            assert_eq!(value["elapsed_world_time_verified"], false);
            assert_eq!(f.records(), 2);
        }
    }

    #[test]
    fn delayed_application_status_observes_clock_without_promoting_or_reissuing_receipt() {
        let f = Fixture::new();
        f.enable();
        let mut ops = BTreeMap::new();
        let request = spec();
        let hours = Cell::new(8.0);
        let queued_target = Cell::new(None);
        let calls = Cell::new(0);
        let receipt = run_with(
            &f.0,
            &mut ops,
            &request,
            "active",
            || Ok(context(hours.get())),
            |target| {
                calls.set(calls.get() + 1);
                queued_target.set(Some(target));
                Ok(())
            },
            || Duration::ZERO,
        )
        .unwrap();
        assert_eq!(receipt["state"], "readback_unverified");
        assert_eq!(receipt["observed_after_hours"], 8.0);
        assert_eq!(receipt["native_request_accepted"], true);
        hours.set(f64::from(queued_target.take().unwrap()));
        assert_eq!(
            run_with(
                &f.0,
                &mut ops,
                &request,
                "active",
                || panic!("replay must not read or dispatch"),
                |_| panic!(),
                || Duration::ZERO,
            )
            .unwrap(),
            receipt
        );
        fs::remove_file(f.0.join(policy::write_gate::FLAG)).unwrap();
        let status =
            policy::clock_control::check(json!({"operation":"fixture"}).as_object().unwrap())
                .unwrap();
        let observed = run_with(
            &f.0,
            &mut ops,
            &status,
            "active",
            || Ok(context(hours.get())),
            |_| panic!(),
            || Duration::ZERO,
        )
        .unwrap();
        assert_eq!(observed["clock_now"]["hours"], 12.0);
        assert_eq!(observed["state"], "readback_unverified");
        assert_eq!(observed["observed_after_hours"], 8.0);
        assert_eq!(observed["world_mutated"], Value::Null);
        assert_eq!(ops["fixture"].outcome, receipt);
        assert_eq!(calls.get(), 1);
        assert_eq!(f.records(), 2);
    }

    #[test]
    fn queued_reply_preserves_results_and_never_cancels_started_clock_work() {
        for started in [false, true] {
            for disconnected in [false, true] {
                let pending = Arc::new(PendingWrite::default());
                if started {
                    assert!(pending.begin());
                }
                let (sender, reply) = mpsc::sync_channel(1);
                let sender = (!disconnected).then_some(sender);
                let error = QueuedClock {
                    pending: pending.clone(),
                    reply,
                }
                .wait(Duration::ZERO)
                .unwrap_err();
                assert_eq!(
                    error,
                    if started {
                        "clock request started; outcome unknown; inspect operation status and journal, never blindly retry"
                    } else {
                        "clock request timed out before execution; queued request cancelled"
                    }
                );
                assert!(!pending.begin());
                assert!(!pending.cancel());
                drop(sender);
            }
        }
        for outcome in [Ok(json!({"state":"fixture"})), Err("native refusal".into())] {
            let pending = Arc::new(PendingWrite::default());
            assert!(pending.begin());
            let (sender, reply) = mpsc::sync_channel(1);
            sender.send(outcome.clone()).unwrap();
            assert_eq!(QueuedClock { pending, reply }.wait(Duration::ZERO), outcome);
        }
    }

    static ADAPTER: Mutex<Option<(ClockControl, Game)>> = Mutex::new(None);
    struct Probe;
    impl enshwick::Mod for Probe {
        fn init(load: &mut enshwick::Load<'_>) -> Result<Self, enshwick::Off> {
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
                inventories: None,
                item_layout: None,
                network_health: None,
                factory_station: None,
                interaction_targets: None,
            };
            let lifecycle = Construction::new(load)?;
            *ADAPTER.lock().unwrap() = Some((ClockControl::new(load.mod_dir()?, lifecycle), game));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.clock-advance"
side = "server""#
    );

    #[test]
    fn scheduled_native_clock_cancels_late_work_and_never_replays_or_crosses_sessions() {
        use enshwick::abi::{DaytimeInfo, Header, Status};
        let f = Fixture::new();
        f.enable();
        let loader = enshwick::test::FakeHost::new()
            .world()
            .mod_dir(f.0.to_str().unwrap())
            .install();
        // SAFETY: this module's declared Probe generates this init export.
        assert_eq!(unsafe { loader.init(enshwick_mod_init) }, Status::OK);
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        const HOUR: i64 = 3_600_000_000_000;
        let info = DaytimeInfo {
            header: Header::of::<DaytimeInfo>(),
            time_ns: 8 * HOUR,
            day_start_ns: 6 * HOUR,
            day_end_ns: 22 * HOUR,
            day_length_ns: 1_800_000_000_000,
            night_length_ns: 720_000_000_000,
            speed: 1.0,
            flags: DaytimeInfo::DAY,
        };
        loader.set_daytime(info);
        let (control, game) = ADAPTER.lock().unwrap().as_ref().unwrap().clone();
        let drive = |request: Spec| {
            let queued = control.queue(game, request, || || Duration::ZERO)?;
            let _ = loader.tick(Duration::from_millis(100));
            queued.wait(Duration::ZERO)
        };
        let mut request = spec();
        request.dry = true;
        request.session = None;
        request.expected_hours = None;
        let dry = drive(request.clone()).unwrap();
        assert_eq!(dry["state"], "dry");
        assert!(!f.0.join("journal.jsonl").exists());
        request.dry = false;
        request.session = Some(dry["session"].as_str().unwrap().into());
        request.expected_hours = dry["expected_hours"].as_f64();
        let timed = control
            .queue(game, request.clone(), || || Duration::ZERO)
            .unwrap()
            .wait(Duration::ZERO)
            .unwrap_err();
        assert!(timed.contains("queued request cancelled"));
        let _ = loader.tick(Duration::from_millis(100));
        assert_eq!(loader.daytime().unwrap().time_ns, 8 * HOUR);
        assert!(!f.0.join("clock-operations").exists());
        let elapsed = Arc::new(Mutex::new(Duration::ZERO));
        let read_elapsed = elapsed.clone();
        let expired = control
            .queue(game, request.clone(), || {
                move || *read_elapsed.lock().unwrap()
            })
            .unwrap();
        *elapsed.lock().unwrap() = clock::WAIT;
        let _ = loader.tick(Duration::from_millis(100));
        assert!(
            expired
                .wait(Duration::ZERO)
                .unwrap_err()
                .contains("deadline expired")
        );
        assert_eq!(loader.daytime().unwrap().time_ns, 8 * HOUR);
        assert!(!f.0.join("clock-operations").exists());
        let queued = control
            .queue(game, request.clone(), || || Duration::ZERO)
            .unwrap();
        assert_eq!(loader.daytime().unwrap().time_ns, 8 * HOUR);
        assert!(!f.0.join("clock-operations").exists());
        let _ = loader.tick(Duration::from_millis(100));
        let result = queued.wait(Duration::ZERO).unwrap();
        assert_eq!(result["readback_near_target"], true);
        assert_eq!(loader.daytime().unwrap().time_ns, 12 * HOUR);
        assert_eq!(f.records(), 2);
        loader.set_daytime(DaytimeInfo {
            time_ns: 13 * HOUR,
            ..info
        });
        assert_eq!(drive(request).unwrap(), result);
        assert_eq!(loader.daytime().unwrap().time_ns, 13 * HOUR);
        assert_eq!(f.records(), 2);
        assert!(loader.raise::<enshwick::events::WorldLoaded>(&()) > 0);
        let status =
            policy::clock_control::check(json!({"operation":"fixture"}).as_object().unwrap())
                .unwrap();
        assert!(drive(status).is_err());
        assert_eq!(loader.daytime().unwrap().time_ns, 13 * HOUR);
    }
}
