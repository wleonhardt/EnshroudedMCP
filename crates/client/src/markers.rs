//! Mod-owned transient map UI, never a vanilla input or server-world executor.
use enshwick::ui::{Marker as NativeMarker, MarkerColor, Ui};
use policy::{
    client_markers::{self as gate, Marker, Spec},
    write_gate,
};
use serde_json::{Value, json};
use std::{path::Path, time::Instant};

#[derive(Debug)]
pub(crate) struct State {
    boot: String,
    epoch: u64,
    revision: u64,
    in_game: bool,
    at: Option<Instant>,
    dirty: bool,
    cleanup_attempted: bool,
    submitted: Vec<Marker>,
    last: Value,
}

impl Default for State {
    fn default() -> Self {
        Self {
            boot: format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            epoch: 0,
            revision: 0,
            in_game: false,
            at: None,
            dirty: false,
            cleanup_attempted: false,
            submitted: Vec::new(),
            last: Value::Null,
        }
    }
}

pub(crate) fn admission(spec: &Spec, dir: Option<&Path>) -> Result<(), String> {
    write_gate::check(
        spec.dry,
        true,
        dir.and_then(|d| write_gate::flag_age(&d.join(write_gate::FLAG))),
        spec.confirm.as_deref(),
    )
}

fn append(dir: Option<&Path>, value: Value, reserve: usize) -> Result<(), String> {
    let dir = dir.ok_or("client mod folder unavailable for marker audit")?;
    let line = json!({"tool":"client_map_markers","result":value}).to_string() + "\n";
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("journal.jsonl"))
        .map_err(|e| e.to_string())?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("client marker journal must be a regular file".into());
    }
    write_gate::journal_budget(meta.len(), line.len(), reserve)?;
    std::io::Write::write_all(&mut file, line.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())
}

pub(crate) fn submit(ui: Option<&Ui>, markers: &[Marker]) -> Result<(), String> {
    let ui = ui.ok_or("client UI unavailable")?;
    let native: Vec<_> = markers
        .iter()
        .map(|m| {
            NativeMarker::new(m.position)
                .label(&m.label)
                .color(MarkerColor::Yellow)
        })
        .collect();
    ui.set_markers(&native).map_err(|e| e.to_string())
}

impl State {
    fn context(&self) -> String {
        format!("{}-{}-{}", self.boot, self.epoch, self.revision)
    }

    pub(crate) fn report(&self) -> Value {
        json!({"context":self.context(),"in_game":self.in_game,"stale":self.at.is_none_or(|at|at.elapsed().as_millis()>1000),
            "last_submitted_markers":gate::encode(&self.submitted),"cleanup_required":self.dirty,
            "last_outcome":self.last,"native_state_readback_available":false,"rendered_verified":false,
            "scope":"this mod's transient client map markers only; submitted cache, not saved markers or server authority"})
    }

    pub(crate) fn observe(
        &mut self,
        in_game: bool,
        dir: Option<&Path>,
        mut apply: impl FnMut(&[Marker]) -> Result<(), String>,
    ) {
        let left = self.in_game && !in_game;
        if self.in_game != in_game {
            self.epoch = self.epoch.saturating_add(1);
        }
        self.in_game = in_game;
        self.at = Some(Instant::now());
        if left && self.dirty && !self.cleanup_attempted {
            self.cleanup_attempted = true;
            let request = self.context();
            self.revision = self.revision.saturating_add(1);
            // Cleanup only removes the admitted transient overlay. Do not leave it behind because
            // its flag expired or audit storage failed. Never retry uncertain cleanup automatically.
            let intent = append(
                dir,
                json!({"kind":"marker_cleanup_intent","request":request}),
                write_gate::JOURNAL_RECORD_BYTES,
            );
            let native = apply(&[]);
            if native.is_ok() {
                self.dirty = false;
                self.submitted.clear();
            }
            let outcome = json!({"kind":"marker_cleanup_outcome","request":request,"native_request_accepted":native.is_ok(),
                "native_error":native.err(),"intent_audit_ok":intent.is_ok(),"rendered_verified":false});
            let recorded = append(dir, outcome.clone(), 0);
            self.last = json!({"state":"leave_cleanup","result":outcome,"audit_ok":intent.is_ok()&&recorded.is_ok()});
        }
    }

    pub(crate) fn run(
        &mut self,
        spec: Spec,
        dir: Option<&Path>,
        mut apply: impl FnMut(&[Marker]) -> Result<(), String>,
    ) -> Result<Value, String> {
        admission(&spec, dir)?;
        let markers = spec
            .markers
            .as_deref()
            .ok_or("marker request payload missing")?;
        gate::context(
            self.in_game,
            self.at.map(|at| at.elapsed().as_millis()),
            markers,
        )?;
        let context = self.context();
        let preview = gate::preview(&context, markers);
        if spec.dry {
            return Ok(
                json!({"state":"dry","preview":preview,"markers":gate::encode(markers),"native_submitted":false,"rendered_verified":false}),
            );
        }
        gate::check_preview(spec.preview.as_deref().unwrap_or(""), &context, markers)?;
        append(
            dir,
            json!({"kind":"marker_intent","request":context,"markers":gate::encode(markers)}),
            write_gate::JOURNAL_RECORD_BYTES,
        )?;
        self.revision = self.revision.saturating_add(1);
        if let Err(error) = admission(&spec, dir) {
            append(
                dir,
                json!({"kind":"marker_outcome","request":context,"native_submitted":false}),
                0,
            )?;
            return Err(error);
        }
        // Retain cleanup responsibility before the native call, including uncertain failures.
        self.dirty |= !markers.is_empty();
        self.cleanup_attempted = markers.is_empty();
        let native = apply(markers);
        if native.is_ok() {
            self.submitted = markers.to_vec();
            self.dirty = !markers.is_empty();
        }
        let result = json!({"state":if native.is_ok(){"submitted"}else{"native_error"},"request":context,
            "native_submitted":true,"native_request_accepted":native.is_ok(),"native_error":native.err(),
            "rendered_verified":false,"context":self.context(),"automatic_retry":false});
        let audit = append(
            dir,
            json!({"kind":"marker_outcome","request":context,"result":result}),
            0,
        );
        self.last = result.clone();
        if let Err(error) = audit {
            self.last["audit_ok"] = json!(false);
            return Err(format!(
                "marker UI may have changed; outcome audit failed; inspect cached status and map, do not blindly retry: {error}"
            ));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::Cell,
        fs,
        path::PathBuf,
        sync::{Arc, Mutex},
        time::Duration,
    };
    struct Folder(PathBuf);
    impl Folder {
        fn new() -> Self {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target")
                .join(format!(
                    "marker-test-{}-{}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
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
    impl Drop for Folder {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn spec() -> Spec {
        gate::check(
            json!({"markers":[{"x":12,"y":30,"z":40,"label":"Fixture"}]})
                .as_object()
                .unwrap(),
        )
        .unwrap()
    }
    fn real(state: &mut State, folder: &Folder) -> Spec {
        let mut spec = spec();
        let dry = state
            .run(spec.clone(), Some(&folder.0), |_| panic!("dry dispatched"))
            .unwrap();
        spec.dry = false;
        spec.confirm = Some("server-write".into());
        spec.preview = Some(dry["preview"].as_str().unwrap().into());
        spec
    }
    #[test]
    fn dry_strict_context_and_durable_intent_precede_submission() {
        let folder = Folder::new();
        let mut state = State::default();
        assert!(
            state
                .run(spec(), None, |_| panic!("stale dispatched"))
                .is_err()
        );
        state.observe(true, None, |_| panic!("initial frame cleared"));
        assert_eq!(
            state
                .run(spec(), None, |_| panic!("ungated dry dispatched"))
                .unwrap()["state"],
            "dry"
        );
        let request = real(&mut state, &folder);
        assert!(!folder.0.join("journal.jsonl").exists());
        let result = state
            .run(request.clone(), Some(&folder.0), |list| {
                assert_eq!(folder.records().len(), 1);
                assert_eq!(list[0].label, "Fixture");
                Ok(())
            })
            .unwrap();
        assert_eq!(result["native_request_accepted"], true);
        assert_eq!(result["rendered_verified"], false);
        let records = folder.records();
        assert_eq!(records.len(), 2);
        assert_eq!(
            records[0]["result"]["request"],
            records[1]["result"]["request"]
        );
        assert!(
            state
                .run(request, Some(&folder.0), |_| panic!(
                    "stale preview dispatched"
                ))
                .is_err()
        );
    }
    #[test]
    fn failed_intent_never_submits_and_outcome_failure_retains_cleanup() {
        let folder = Folder::new();
        let mut state = State::default();
        state.observe(true, None, |_| Ok(()));
        let request = real(&mut state, &folder);
        fs::create_dir(folder.0.join("journal.jsonl")).unwrap();
        assert!(
            state
                .run(request.clone(), Some(&folder.0), |_| panic!(
                    "failed intent dispatched"
                ))
                .is_err()
        );
        fs::remove_dir(folder.0.join("journal.jsonl")).unwrap();
        let error = state
            .run(request, Some(&folder.0), |_| {
                fs::remove_file(folder.0.join("journal.jsonl")).unwrap();
                fs::create_dir(folder.0.join("journal.jsonl")).unwrap();
                Ok(())
            })
            .unwrap_err();
        assert!(error.contains("may have changed"));
        assert_eq!(state.report()["cleanup_required"], true);
        fs::remove_file(folder.0.join(write_gate::FLAG)).unwrap();
        let cleared = Cell::new(false);
        state.observe(false, Some(&folder.0), |list| {
            assert!(list.is_empty());
            cleared.set(true);
            Ok(())
        });
        assert!(cleared.get());
        assert_eq!(state.report()["cleanup_required"], false);
        assert_eq!(state.report()["last_outcome"]["audit_ok"], false);
    }
    #[test]
    fn uncertain_cleanup_is_not_retried_across_later_transitions() {
        let folder = Folder::new();
        let mut state = State::default();
        state.observe(true, None, |_| Ok(()));
        let request = real(&mut state, &folder);
        state.run(request, Some(&folder.0), |_| Ok(())).unwrap();
        fs::remove_file(folder.0.join(write_gate::FLAG)).unwrap();
        let calls = Cell::new(0);
        state.observe(false, Some(&folder.0), |list| {
            assert!(list.is_empty());
            calls.set(calls.get() + 1);
            Err("unavailable".into())
        });
        state.observe(false, Some(&folder.0), |_| {
            panic!("same-frame cleanup retry")
        });
        state.observe(true, Some(&folder.0), |_| {
            panic!("new-session cleanup retry")
        });
        state.observe(false, Some(&folder.0), |_| {
            panic!("later-leave cleanup retry")
        });
        assert_eq!(calls.get(), 1);
        assert_eq!(state.report()["cleanup_required"], true);
    }

    static TOOLS: Mutex<Option<(Arc<crate::ClientTools>, Ui)>> = Mutex::new(None);
    struct Probe;
    impl enshwick::Mod for Probe {
        fn init(load: &mut enshwick::Load<'_>) -> Result<Self, enshwick::Off> {
            let ui = load.ui()?;
            let client = crate::ResearchClient::init(load)?;
            *TOOLS.lock().unwrap() = Some((Arc::new(client.tools), ui));
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(
        Probe,
        r#"id = "test.client-markers"
side = "both""#
    );
    #[test]
    fn production_frame_queue_native_submission_cancellation_gate_and_cleanup() {
        use policy::mcp::Tools;
        let folder = Folder::new();
        let loader = enshwick::test::FakeHost::new()
            .side(enshwick::abi::Side::CLIENT)
            .mod_dir(folder.0.to_str().unwrap())
            .install();
        // SAFETY: exports are generated by this module's declared test probe.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let (tools, ui) = TOOLS.lock().unwrap().as_ref().unwrap().clone();
        let _ = loader.draw_frame();
        let arguments = json!({"markers":[{"x":12,"y":30,"z":40,"label":"Fixture"}]});
        let drive = |args: Value| {
            let tools = tools.clone();
            let worker = std::thread::spawn(move || {
                tools.call("client_map_markers", args.as_object().unwrap())
            });
            let deadline = Instant::now() + Duration::from_secs(3);
            while !worker.is_finished() {
                assert!(Instant::now() < deadline);
                let _ = loader.draw_frame();
                std::thread::sleep(Duration::from_millis(1));
            }
            worker.join().unwrap()
        };
        let dry = drive(arguments.clone()).unwrap();
        assert!(loader.markers().is_empty());
        assert!(!folder.0.join("journal.jsonl").exists());
        let mut real = arguments.clone();
        real["dry"] = json!(false);
        real["confirm"] = json!("server-write");
        real["preview"] = dry["preview"].clone();
        let cancelled_tools = tools.clone();
        let cancelled = real.clone();
        assert!(
            std::thread::spawn(
                move || cancelled_tools.call("client_map_markers", cancelled.as_object().unwrap())
            )
            .join()
            .unwrap()
            .unwrap_err()
            .contains("cancelled")
        );
        let _ = loader.draw_frame();
        assert!(loader.markers().is_empty());
        assert!(!folder.0.join("journal.jsonl").exists());
        let revoked_tools = tools.clone();
        let revoked = real.clone();
        let worker = std::thread::spawn(move || {
            revoked_tools.call("client_map_markers", revoked.as_object().unwrap())
        });
        let deadline = Instant::now() + Duration::from_secs(1);
        while tools.shared.pending.lock().unwrap().is_none() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        fs::remove_file(folder.0.join(write_gate::FLAG)).unwrap();
        let _ = loader.draw_frame();
        assert!(worker.join().unwrap().is_err());
        assert!(loader.markers().is_empty());
        assert!(!folder.0.join("journal.jsonl").exists());
        fs::write(folder.0.join(write_gate::FLAG), []).unwrap();
        match drive(real.clone()) {
            Ok(value) => assert_eq!(value["native_request_accepted"], true),
            Err(error) => assert!(
                error.starts_with("client UI request started") && error.contains("outcome unknown"),
                "{error}"
            ),
        }
        let observed = tools
            .call("client_map_markers", &serde_json::Map::new())
            .unwrap();
        assert_eq!(observed["last_outcome"]["native_request_accepted"], true);
        assert_eq!(
            folder.records()[1]["result"]["result"],
            observed["last_outcome"]
        );
        let native = loader.markers();
        assert_eq!(native.len(), 1);
        assert_eq!(native[0].label, "Fixture");
        assert_eq!(native[0].position, [12.0, 30.0, 40.0]);
        assert_eq!(native[0].marker_type, 0);
        assert_eq!(folder.records().len(), 2);
        assert!(drive(real).is_err());
        fs::remove_file(folder.0.join(write_gate::FLAG)).unwrap();
        tools
            .shared
            .markers
            .lock()
            .unwrap()
            .observe(false, Some(&folder.0), |list| submit(Some(&ui), list));
        assert!(loader.markers().is_empty());
        assert_eq!(folder.records().len(), 4);
        let report = tools
            .call("client_map_markers", &serde_json::Map::new())
            .unwrap();
        assert_eq!(report["rendered_verified"], false);
        assert_eq!(report["cleanup_required"], false);
    }

    #[test]
    fn production_frame_queue_retains_audited_outcome_when_reply_is_lost() {
        use policy::mcp::Tools;
        let folder = Folder::new();
        let loader = enshwick::test::FakeHost::new()
            .side(enshwick::abi::Side::CLIENT)
            .mod_dir(folder.0.to_str().unwrap())
            .install();
        // SAFETY: exports are generated by this module's declared test probe.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let (tools, ui) = TOOLS.lock().unwrap().as_ref().unwrap().clone();
        let _ = loader.draw_frame();
        let request = real(&mut tools.shared.markers.lock().unwrap(), &folder);
        let (reply, receiver) = std::sync::mpsc::sync_channel(1);
        drop(receiver); // A lost caller cannot receive the successful submission.
        *tools.shared.pending.lock().unwrap() = Some(crate::Pending {
            request: crate::Request::Markers(request.clone()),
            cancel: Arc::new(policy::write_gate::PendingWrite::default()),
            reply,
        });
        let _ = loader.draw_frame();
        assert_eq!(loader.markers().len(), 1);
        let observed = tools
            .call("client_map_markers", &serde_json::Map::new())
            .unwrap();
        assert_eq!(observed["last_outcome"]["native_request_accepted"], true);
        assert_eq!(
            folder.records()[1]["result"]["result"],
            observed["last_outcome"]
        );
        assert_eq!(observed["cleanup_required"], true);
        assert_eq!(folder.records().len(), 2);
        assert!(
            tools
                .shared
                .markers
                .lock()
                .unwrap()
                .run(request, Some(&folder.0), |_| panic!("lost reply retried"))
                .is_err()
        );
        assert_eq!(folder.records().len(), 2);
        fs::remove_file(folder.0.join(write_gate::FLAG)).unwrap();
        tools
            .shared
            .markers
            .lock()
            .unwrap()
            .observe(false, Some(&folder.0), |list| submit(Some(&ui), list));
        assert!(loader.markers().is_empty());
        assert_eq!(folder.records().len(), 4);
    }
}
