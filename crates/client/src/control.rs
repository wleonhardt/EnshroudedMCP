//! Client-frame admission; the SDK executes commands later in its client phase.
use enshwick::input::{
    ClientCommand, CommandAction, CommandId, CommandState, Input, SnapshotPart, SnapshotState,
};
use policy::{client_control as gate, write_gate};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};

struct Entry {
    request: Value,
    id: Option<CommandId>,
    result: Value,
    terminal: bool,
    cancel_attempted: bool,
    observed_at: Option<Instant>,
}
impl Entry {
    fn outcome(&self) -> Value {
        let mut result = self.result.clone();
        result["status_observation"] =
            gate::status_observation(self.observed_at.map(|at| at.elapsed()), Some(self.terminal));
        result
    }
}
pub(crate) struct State {
    boot: String,
    operations: BTreeMap<String, Entry>,
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
            operations: BTreeMap::new(),
        }
    }
}
pub(crate) fn admission(
    dry: bool,
    confirm: Option<&str>,
    dir: Option<&Path>,
) -> Result<(), String> {
    write_gate::check(
        dry,
        true,
        dir.and_then(|d| write_gate::flag_age(&d.join(write_gate::FLAG))),
        confirm,
    )
}
fn journal_size(dir: &Path) -> Result<u64, String> {
    match fs::metadata(dir.join("journal.jsonl")) {
        Ok(meta) if meta.is_file() => Ok(meta.len()),
        Ok(_) => Err("client command journal must be a regular file".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error.to_string()),
    }
}
fn append(dir: &Path, value: &Value) -> Result<(), String> {
    let line = json!({"tool":"client_command","result":value}).to_string() + "\n";
    write_gate::journal_budget(journal_size(dir)?, line.len(), 0)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("journal.jsonl"))
        .map_err(|e| e.to_string())?;
    file.write_all(line.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())
}
fn path(dir: &Path, operation: &str) -> std::path::PathBuf {
    dir.join("client-commands")
        .join(format!("command-{operation}.json"))
}
fn reserve(dir: &Path, operation: &str, intent: &Value) -> Result<(), String> {
    let namespace = dir.join("client-commands");
    match fs::read_dir(&namespace) {
        Ok(entries) => {
            for (index, entry) in entries.enumerate() {
                entry.map_err(|e| e.to_string())?;
                gate::capacity(index + 1)?;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.to_string()),
    }
    gate::journal_headroom(journal_size(dir)?, 3)?;
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
    append(dir, intent)
}
fn context(input: Input) -> Result<gate::Context, String> {
    let snapshot = input.client_snapshot().map_err(|e| e.to_string())?;
    Ok(gate::Context {
        fresh: snapshot.state() == SnapshotState::Fresh,
        age_ms: snapshot.age_ms(),
        session: snapshot.session(),
        screens_read: snapshot.reads(SnapshotPart::Screens),
        screen: snapshot.hud_screen().map(str::to_owned),
        tab_read: snapshot.reads(SnapshotPart::MenuTab),
        tab: snapshot.menu_tab().map(str::to_owned),
    })
}
fn native(action: &gate::Action) -> CommandAction {
    match action {
        gate::Action::Open(target) => CommandAction::OpenScreen(target.clone()),
        gate::Action::Close => CommandAction::CloseScreen,
        gate::Action::Tab(target) => CommandAction::ChooseTab(target.clone()),
        gate::Action::Look(yaw, pitch) => CommandAction::Look {
            yaw: *yaw,
            pitch: *pitch,
        },
        gate::Action::Move(direction, hold_ms) => CommandAction::Move {
            direction: *direction,
            hold_ms: *hold_ms,
        },
        gate::Action::Recipe(id) => CommandAction::HighlightRecipe(enshwick::RecipeId(*id)),
    }
}
impl State {
    pub(crate) fn inventory(&self, dir: Option<&Path>) -> Result<Value, String> {
        let mut operations: BTreeSet<_> = self.operations.keys().cloned().collect();
        if let Some(dir) = dir {
            match fs::read_dir(dir.join("client-commands")) {
                Ok(entries) => {
                    for (index, entry) in entries.enumerate() {
                        gate::capacity(index)?;
                        let entry = entry.map_err(|error| error.to_string())?;
                        if !entry
                            .file_type()
                            .map_err(|error| error.to_string())?
                            .is_file()
                        {
                            return Err("client command reservations must be regular files".into());
                        }
                        let name = entry.file_name();
                        operations.insert(gate::reserved_operation(
                            name.to_str()
                                .ok_or("invalid client command reservation name")?,
                        )?);
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => return Err(error.to_string()),
            }
        }
        let commands = operations
            .iter()
            .map(|operation| self.report(operation, dir))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(
            json!({"commands":commands,"commands_submitted":false,"native_identity_reconstructed":false,
            "automatic_retry":false,"scope":"retained client operations and durable caller identities",
            "max_records":2*gate::MAX_OPERATIONS}),
        )
    }
    pub(crate) fn report(&self, operation: &str, dir: Option<&Path>) -> Result<Value, String> {
        if let Some(entry) = self.operations.get(operation) {
            return Ok(entry.outcome());
        }
        if dir.is_some_and(|dir| path(dir, operation).exists()) {
            return Ok(
                json!({"operation":operation,"state":"prior_boot_unavailable","automatic_retry":false,"native_identity_reconstructed":false,
                    "status_observation":gate::status_observation(None,None)}),
            );
        }
        Err("unknown client command operation".into())
    }
    pub(crate) fn run(
        &mut self,
        spec: gate::Spec,
        input: Option<Input>,
        dir: Option<&Path>,
        queued: Instant,
    ) -> Result<Value, String> {
        admission(spec.dry, spec.confirm.as_deref(), dir)?;
        let request = gate::payload(&spec.action);
        if let Some(entry) = self.operations.get(&spec.operation) {
            if entry.request != request {
                return Err("client operation payload conflicts with retained identity".into());
            }
            return Ok(entry.outcome());
        }
        if dir.is_some_and(|dir| path(dir, &spec.operation).exists()) {
            return Err(
                "client operation belongs to a prior boot; never reconstruct or redispatch it"
                    .into(),
            );
        }
        let input = input.ok_or("native client command API unavailable")?;
        gate::deadline(queued.elapsed())?;
        let ctx = context(input)?;
        gate::applicable(&ctx, &spec.action)?;
        let preview = gate::preview(&self.boot, &spec, &ctx);
        if spec.dry {
            return Ok(
                json!({"state":"dry","operation":spec.operation,"preview":preview,"request":request,"session":ctx.session,"native_submitted":false,"rendered_verified":false}),
            );
        }
        gate::exact(spec.preview.as_deref(), &preview)?;
        gate::capacity(self.operations.len())?;
        let dir = dir.ok_or("client mod folder unavailable")?;
        let intent = json!({"kind":"client_command_intent","operation":spec.operation,"boot":self.boot,"session":ctx.session,"request":request});
        let mut entry = Entry {
            request,
            id: None,
            result: json!({"state":"reserved","operation":spec.operation,"session":ctx.session,"native_submitted":false,"automatic_retry":false,"rendered_verified":false,"arrival_verified":false}),
            terminal: true,
            cancel_attempted: false,
            observed_at: None,
        };
        let result = (|| {
            reserve(dir, &spec.operation, &intent)?;
            admission(false, spec.confirm.as_deref(), Some(dir))?;
            gate::deadline(queued.elapsed())?;
            let now = context(input)?;
            gate::applicable(&now, &spec.action)?;
            gate::exact(Some(&preview), &gate::preview(&self.boot, &spec, &now))?;
            let command = ClientCommand::new(native(&spec.action), now.session)
                .on_screen(now.screen.as_deref().unwrap());
            // Snapshot collection can outlive the earlier admission checks.
            admission(false, spec.confirm.as_deref(), Some(dir))?;
            gate::deadline(queued.elapsed())?;
            // Record uncertainty before the call. Never reconstruct an id or redispatch this identity.
            entry.result["native_submitted"] = json!(true);
            let id = input.client_command(&command).map_err(|e| e.to_string())?;
            entry.id = Some(id);
            entry.terminal = false;
            entry.result["state"] = json!("queued");
            entry.result["native_state"] = json!("pending");
            Ok::<(), String>(())
        })();
        if let Err(error) = result {
            entry.result["state"] = json!("refused_or_unknown");
            entry.result["error"] = json!(error);
        }
        if let Err(error) = append(
            dir,
            &json!({"kind":"client_command_submission","operation":spec.operation,"result":entry.result}),
        ) {
            entry.result["audit_error"] = json!(error);
        }
        let result = entry.outcome();
        self.operations.insert(spec.operation, entry);
        Ok(result)
    }
    pub(crate) fn observe(&mut self, input: Option<Input>, dir: Option<&Path>) {
        let Some(input) = input else {
            return;
        };
        for (operation, entry) in &mut self.operations {
            if entry.terminal {
                continue;
            }
            let Some(id) = entry.id else {
                continue;
            };
            let status = input.command_status(id);
            entry.observed_at = Some(Instant::now());
            match status {
                Ok(status) => {
                    let (state, terminal) = match status.state {
                        CommandState::Pending => ("pending", false),
                        CommandState::Applied => ("applied", false),
                        CommandState::Observed => ("observed", true),
                        CommandState::Cancelled(reason) => {
                            entry.result["cancel_reason"] = json!(format!("{reason:?}"));
                            ("cancelled", true)
                        }
                        CommandState::Unavailable => ("unavailable", true),
                        _ => ("unknown", true),
                    };
                    entry.result["native_state"] = json!(state);
                    entry.result["native_age_ms"] = json!(status.age_ms);
                    entry.terminal = terminal;
                }
                Err(error) => {
                    entry.result["native_state"] = json!("unknown");
                    entry.result["native_age_ms"] = Value::Null;
                    entry.result["native_error"] = json!(error.to_string());
                    entry.terminal = true;
                }
            }
            if entry.terminal {
                entry.result["state"] = json!("finished_or_unknown");
                let audit=dir.ok_or_else(||"client command audit folder unavailable".to_owned()).and_then(|dir|append(dir,&json!({"kind":"client_command_terminal","operation":operation,"result":entry.result})));
                if let Err(error) = audit {
                    entry.result["audit_error"] = json!(error);
                }
            }
        }
    }
    pub(crate) fn cancel(
        &mut self,
        spec: gate::CancelSpec,
        input: Option<Input>,
        dir: Option<&Path>,
        queued: Instant,
    ) -> Result<Value, String> {
        let operation = spec.operation.as_str();
        let confirm = spec.confirm.as_deref();
        admission(spec.dry, confirm, dir)?;
        let entry = self
            .operations
            .get_mut(operation)
            .ok_or("client cancellation requires a retained operation; no ID reconstruction")?;
        let token = json!({"boot":self.boot,"operation":operation,"cancel":true}).to_string();
        if spec.dry {
            return Ok(
                json!({"state":"dry_cancel","operation":operation,"preview":token,"last_outcome":entry.outcome(),"native_submitted":false}),
            );
        }
        gate::exact(spec.preview.as_deref(), &token)?;
        if entry.terminal || entry.cancel_attempted {
            return Ok(entry.outcome());
        }
        let dir = dir.ok_or("client mod folder unavailable")?;
        let input = input.ok_or("native client command API unavailable")?;
        let id = entry
            .id
            .ok_or("client operation has no native command identity")?;
        gate::deadline(queued.elapsed())?;
        gate::journal_headroom(journal_size(dir)?, 2)?;
        append(
            dir,
            &json!({"kind":"client_command_cancel_intent","operation":operation}),
        )?;
        admission(false, confirm, Some(dir))?;
        gate::deadline(queued.elapsed())?;
        entry.cancel_attempted = true;
        let result = input.cancel_command(id);
        entry.result["cancel_request_accepted"] = json!(result.is_ok());
        entry.result["cancel_attempted"] = json!(true);
        if let Err(error) = result {
            entry.result["cancel_error"] = json!(error.to_string());
        }
        if let Err(error) = append(
            dir,
            &json!({"kind":"client_command_cancel_submission","operation":operation,"result":entry.result}),
        ) {
            entry.result["audit_error"] = json!(error);
        }
        Ok(entry.outcome())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enshwick::abi::{ClientCommandStatus as Status, ClientSnapshot as Raw, Side, Str};
    use enshwick::{Load, Mod, Off};
    use std::sync::Mutex;
    static INPUT: Mutex<Option<Input>> = Mutex::new(None);
    struct Driver;
    impl Mod for Driver {
        fn init(load: &mut Load<'_>) -> Result<Self, Off> {
            *INPUT.lock().unwrap() = Some(load.input()?);
            Ok(Self)
        }
    }
    enshwick::test::declare_mod!(Driver, "id = \"test.native-control\"\nside = \"both\"");
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            let parent =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/client-control-tests");
            fs::create_dir_all(&parent).unwrap();
            let dir = parent.join(format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&dir).unwrap();
            Self(dir)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            if std::thread::panicking() {
                eprintln!(
                    "client fixture cleanup skipped during panic: {}",
                    self.0.display()
                );
                return;
            }
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn fixture_unwind_retains_exact_evidence_while_success_cleans_up() {
        let completed = Fixture::new();
        let parent = completed.0.parent().unwrap().canonicalize().unwrap();
        let completed_path = completed.0.canonicalize().unwrap();
        assert_eq!(completed_path.parent(), Some(parent.as_path()));
        reserve(&completed.0, "completed", &json!({"operation":"completed"})).unwrap();
        drop(completed);
        assert!(!completed_path.exists());

        let failed = Fixture::new();
        let dir = failed.0.canonicalize().unwrap();
        assert_eq!(dir.parent(), Some(parent.as_path()));
        let operation = "retained-evidence";
        reserve(
            &dir,
            operation,
            &json!({"operation":operation,"synthetic":true}),
        )
        .unwrap();
        let marker = path(&dir, operation);
        let journal = dir.join("journal.jsonl");
        let marker_before = fs::read(&marker).unwrap();
        let journal_before = fs::read(&journal).unwrap();
        let result = std::panic::catch_unwind(move || {
            let _fixture = failed;
            panic!("controlled fixture unwind");
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&marker).unwrap(), marker_before);
        assert_eq!(fs::read(&journal).unwrap(), journal_before);
        let mut names: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        names.sort();
        assert_eq!(names, vec!["client-commands", "journal.jsonl"]);
        assert_eq!(
            fs::read_dir(dir.join("client-commands")).unwrap().count(),
            1
        );
        fs::remove_file(marker).unwrap();
        fs::remove_dir(dir.join("client-commands")).unwrap();
        fs::remove_file(journal).unwrap();
        fs::remove_dir(dir).unwrap();
    }
    fn retained_outcome(mut value: Value) -> Value {
        value.as_object_mut().unwrap().remove("status_observation");
        value
    }
    fn spec(value: Value) -> gate::Spec {
        gate::check(value.as_object().unwrap()).unwrap()
    }
    fn dry(operation: &str) -> gate::Spec {
        spec(json!({"operation":operation,"action":"open_screen","target":"Map"}))
    }
    fn real(mut spec: gate::Spec, preview: &Value) -> gate::Spec {
        spec.dry = false;
        spec.preview = Some(preview["preview"].as_str().unwrap().into());
        spec.confirm = Some(write_gate::CONFIRM.into());
        spec
    }
    fn snapshot(session: u64, screen: &'static str, tab: &'static str) -> Raw {
        Raw {
            state: Raw::FRESH,
            age_ms: 0,
            session,
            available: Raw::SCREENS | Raw::MENU_TAB,
            hud_screen: Str::new(screen),
            menu_tab: Str::new(tab),
            ..Raw::EMPTY
        }
    }
    #[test]
    fn native_dispatch_replay_cancel_reload_and_audit_failures_never_duplicate_input() {
        let loader = enshwick::test::FakeHost::new()
            .side(Side::CLIENT)
            .player_input()
            .install();
        // SAFETY: these are this test module's framework-generated exports.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let input = *INPUT.lock().unwrap();
        let input_value = input.unwrap();
        loader.set_client_snapshot(snapshot(1, "InGame", ""));
        let fixture = Fixture::new();
        let dir = Some(fixture.0.as_path());
        let mut state = State::default();
        let preview = state.run(dry("open"), input, dir, Instant::now()).unwrap();
        assert_eq!(preview["state"], "dry");
        assert!(loader.client_commands().is_empty());
        assert!(!fixture.0.join("journal.jsonl").exists());
        let write = real(dry("open"), &preview);
        assert!(
            state
                .run(write.clone(), input, dir, Instant::now())
                .is_err()
        );
        fs::write(fixture.0.join(write_gate::FLAG), b"").unwrap();
        let queued = state
            .run(write.clone(), input, dir, Instant::now())
            .unwrap();
        assert_eq!(queued["state"], "queued", "synthetic submission: {queued}");
        assert_eq!(loader.client_commands().len(), 1);
        let journal_before_inventory = fs::read(fixture.0.join("journal.jsonl")).unwrap();
        let inventory = state.inventory(dir).unwrap();
        assert_eq!(inventory["commands"].as_array().unwrap().len(), 1);
        assert_eq!(inventory["commands"][0], queued);
        assert_eq!(inventory["commands_submitted"], false);
        assert_eq!(
            fs::read(fixture.0.join("journal.jsonl")).unwrap(),
            journal_before_inventory
        );
        assert_eq!(loader.client_commands().len(), 1);
        assert_eq!(
            state
                .run(write.clone(), input, dir, Instant::now())
                .unwrap(),
            queued
        );
        assert_eq!(loader.client_commands().len(), 1);
        let mut conflict = write.clone();
        conflict.action = gate::Action::Close;
        assert!(state.run(conflict, input, dir, Instant::now()).is_err());
        let cancel_preview = state
            .cancel(
                gate::cancel(json!({"operation":"open"}).as_object().unwrap()).unwrap(),
                input,
                dir,
                Instant::now(),
            )
            .unwrap();
        let cancel=gate::cancel(json!({"operation":"open","dry":false,"preview":cancel_preview["preview"],"confirm":write_gate::CONFIRM}).as_object().unwrap()).unwrap();
        state
            .cancel(cancel.clone(), input, dir, Instant::now())
            .unwrap();
        state.cancel(cancel, input, dir, Instant::now()).unwrap();
        assert_eq!(
            input_value.command_status(CommandId::new(1)).unwrap().state,
            CommandState::Cancelled(enshwick::input::CancelReason::ByMod)
        );
        state.observe(input, dir);
        let done = state.report("open", dir).unwrap();
        assert_eq!(done["native_state"], "cancelled");
        assert_eq!(done["cancel_attempted"], true);
        assert_eq!(done["arrival_verified"], false);
        assert_eq!(loader.client_commands().len(), 1);
        let mut reopened = State::default();
        assert_eq!(
            reopened.report("open", dir).unwrap()["state"],
            "prior_boot_unavailable"
        );
        let inventory = reopened.inventory(dir).unwrap();
        assert_eq!(inventory["commands"].as_array().unwrap().len(), 1);
        assert_eq!(
            inventory["commands"][0],
            reopened.report("open", dir).unwrap()
        );
        assert_eq!(inventory["native_identity_reconstructed"], false);
        assert!(reopened.run(write, input, dir, Instant::now()).is_err());
        let preview = state.run(dry("stale"), input, dir, Instant::now()).unwrap();
        loader.set_client_snapshot(snapshot(2, "InGame", ""));
        assert!(
            state
                .run(real(dry("stale"), &preview), input, dir, Instant::now())
                .is_err()
        );
        assert!(!path(&fixture.0, "stale").exists());
        // Each supported family reaches only its typed SDK action; observed is not arrival/rendering.
        for (index, action) in [
            gate::Action::Close,
            gate::Action::Look(0.2, -0.1),
            gate::Action::Move([0.0, 1.0], 250),
            gate::Action::Tab("Map".into()),
            gate::Action::Recipe(73),
        ]
        .into_iter()
        .enumerate()
        {
            let (screen, tab) = match action {
                gate::Action::Tab(_) | gate::Action::Recipe(_) => ("InGameMenu", "PlayerCrafting"),
                _ => ("InGame", ""),
            };
            loader.set_client_snapshot(snapshot(2, screen, tab));
            let mut request = dry(&format!("family-{index}"));
            request.action = action;
            let preview = state
                .run(request.clone(), input, dir, Instant::now())
                .unwrap();
            let request = real(request, &preview);
            let result = state
                .run(request.clone(), input, dir, Instant::now())
                .unwrap();
            assert_eq!(result["state"], "queued");
            let id = loader.client_commands().last().unwrap().0;
            loader.set_command_status(id, Status::APPLIED, 0);
            state.observe(input, dir);
            let journal = fixture.0.join("journal.jsonl");
            let saved = if index == 0 {
                let bytes = fs::read(&journal).unwrap();
                fs::remove_file(&journal).unwrap();
                fs::create_dir(&journal).unwrap();
                Some(bytes)
            } else {
                None
            };
            loader.set_command_status(id, Status::OBSERVED, 0);
            state.observe(input, dir);
            let report = state.report(&format!("family-{index}"), dir).unwrap();
            assert_eq!(report["native_state"], "observed");
            assert_eq!(report["rendered_verified"], false);
            if let Some(saved) = saved {
                assert!(report["audit_error"].is_string());
                let before = loader.client_commands().len();
                assert_eq!(
                    retained_outcome(state.run(request, input, dir, Instant::now()).unwrap()),
                    retained_outcome(report)
                );
                assert_eq!(loader.client_commands().len(), before);
                fs::remove_dir(&journal).unwrap();
                fs::write(&journal, saved).unwrap();
            }
        }
        let before = loader.client_commands().len();
        loader.set_client_snapshot(snapshot(2, "InGame", ""));
        let request = dry("blocked-audit");
        let preview = state
            .run(request.clone(), input, dir, Instant::now())
            .unwrap();
        let journal = fixture.0.join("journal.jsonl");
        fs::remove_file(&journal).unwrap();
        fs::create_dir(&journal).unwrap();
        let blocked = state
            .run(real(request, &preview), input, dir, Instant::now())
            .unwrap();
        assert_eq!(blocked["native_submitted"], false);
        assert!(blocked["audit_error"].is_string());
        assert_eq!(loader.client_commands().len(), before);
        assert!(
            state
                .run(dry("deadline"), input, dir, Instant::now() - gate::WAIT)
                .is_err()
        );
    }

    #[test]
    fn durable_inventory_is_bounded_sorted_read_only_and_never_decodes_old_native_ids() {
        let fixture = Fixture::new();
        let dir = Some(fixture.0.as_path());
        let state = State::default();
        assert_eq!(state.inventory(None).unwrap()["commands"], json!([]));
        assert_eq!(state.inventory(dir).unwrap()["commands"], json!([]));
        let namespace = fixture.0.join("client-commands");
        fs::create_dir(&namespace).unwrap();
        // Reservation contents are deliberately unreadable as JSON. Only caller identities are recovered.
        for operation in ["z-last", "a-first"] {
            fs::write(path(&fixture.0, operation), b"not a native command record").unwrap();
        }
        let inventory = state.inventory(dir).unwrap();
        assert_eq!(inventory["commands"][0]["operation"], "a-first");
        assert_eq!(inventory["commands"][1]["operation"], "z-last");
        assert!(
            inventory["commands"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row["state"] == "prior_boot_unavailable"
                    && row["automatic_retry"] == false)
        );
        assert!(!fixture.0.join("journal.jsonl").exists());
        fs::create_dir(path(&fixture.0, "directory")).unwrap();
        assert!(state.inventory(dir).is_err());
        fs::remove_dir(path(&fixture.0, "directory")).unwrap();
        let invalid = namespace.join("command-INVALID.json");
        fs::write(&invalid, b"").unwrap();
        assert!(state.inventory(dir).is_err());
        fs::remove_file(invalid).unwrap();
        for index in 2..gate::MAX_OPERATIONS {
            fs::write(path(&fixture.0, &format!("entry-{index}")), b"").unwrap();
        }
        assert_eq!(
            state.inventory(dir).unwrap()["commands"]
                .as_array()
                .unwrap()
                .len(),
            gate::MAX_OPERATIONS
        );
        fs::write(path(&fixture.0, "overflow"), b"").unwrap();
        assert!(state.inventory(dir).is_err());
    }

    #[test]
    fn native_cancellation_and_unavailability_remain_terminal_without_redispatch() {
        let loader = enshwick::test::FakeHost::new()
            .side(Side::CLIENT)
            .player_input()
            .install();
        // SAFETY: these are this test module's framework-generated exports.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let input = *INPUT.lock().unwrap();
        loader.set_client_snapshot(snapshot(7, "InGame", ""));
        let fixture = Fixture::new();
        fs::write(fixture.0.join(write_gate::FLAG), b"").unwrap();
        let dir = Some(fixture.0.as_path());
        let mut state = State::default();
        for (reason, expected) in [
            (Status::SESSION_CHANGED, "SessionChanged"),
            (Status::OTHER_SCREEN, "OtherScreen"),
            (Status::PLAYER_INPUT, "PlayerInput"),
            (Status::BY_MOD, "ByMod"),
            (Status::TIMED_OUT, "TimedOut"),
            (Status::INPUT_BLOCKED, "InputBlocked"),
            (Status::MOD_OFF, "ModOff"),
            (999, "Other(999)"),
            (0, ""),
        ] {
            let operation = format!("terminal-{reason}");
            let request = dry(&operation);
            let preview = state
                .run(request.clone(), input, dir, Instant::now())
                .unwrap();
            let request = real(request, &preview);
            let queued = state
                .run(request.clone(), input, dir, Instant::now())
                .unwrap();
            assert_eq!(
                queued["state"], "queued",
                "synthetic terminal submission: {queued}"
            );
            let commands = loader.client_commands();
            let id = commands.last().unwrap().0;
            loader.set_command_status(id, Status::APPLIED, 0);
            state.observe(input, dir);
            assert_eq!(state.report(&operation, dir).unwrap()["state"], "queued");
            loader.set_command_status(
                id,
                if reason == 0 {
                    Status::UNAVAILABLE
                } else {
                    Status::CANCELLED
                },
                reason,
            );
            state.observe(input, dir);
            let report = state.report(&operation, dir).unwrap();
            assert_eq!(report["state"], "finished_or_unknown");
            assert_eq!(
                report["native_state"],
                if reason == 0 {
                    "unavailable"
                } else {
                    "cancelled"
                }
            );
            if reason != 0 {
                assert_eq!(report["cancel_reason"], expected);
            }
            assert_eq!(report["rendered_verified"], false);
            assert_eq!(report["arrival_verified"], false);
            let journal = fs::read(fixture.0.join("journal.jsonl")).unwrap();
            state.observe(input, dir);
            assert_eq!(
                retained_outcome(state.run(request, input, dir, Instant::now()).unwrap()),
                retained_outcome(report)
            );
            assert_eq!(loader.client_commands(), commands);
            assert_eq!(fs::read(fixture.0.join("journal.jsonl")).unwrap(), journal);
        }
    }

    #[test]
    fn cancellation_intent_fault_preserves_pending_command_and_attempt_ownership() {
        let loader = enshwick::test::FakeHost::new()
            .side(Side::CLIENT)
            .player_input()
            .install();
        // SAFETY: these are this test module's framework-generated exports.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let input = *INPUT.lock().unwrap();
        let native = input.unwrap();
        native.client_snapshot().unwrap();
        let _ = loader.draw_frame();
        loader.set_client_snapshot(snapshot(7, "InGame", ""));
        let fixture = Fixture::new();
        let dir = Some(fixture.0.as_path());
        fs::write(fixture.0.join(write_gate::FLAG), b"").unwrap();
        let mut state = State::default();
        let preview = state
            .run(dry("cancel-fault"), input, dir, Instant::now())
            .unwrap();
        let submitted = state
            .run(
                real(dry("cancel-fault"), &preview),
                input,
                dir,
                Instant::now(),
            )
            .unwrap();
        assert_eq!(submitted["native_state"], "pending");
        let native_id = loader.client_commands()[0].0;
        let id = CommandId::new(native_id);
        let preview = state
            .cancel(
                gate::cancel(json!({"operation":"cancel-fault"}).as_object().unwrap()).unwrap(),
                input,
                dir,
                Instant::now(),
            )
            .unwrap();
        let cancel = gate::cancel(json!({"operation":"cancel-fault","dry":false,"preview":preview["preview"],"confirm":write_gate::CONFIRM}).as_object().unwrap()).unwrap();
        let journal = fixture.0.join("journal.jsonl");
        let original = fs::read(&journal).unwrap();
        let reservation = fs::read(path(&fixture.0, "cancel-fault")).unwrap();
        // A non-file audit path must refuse before touching the pending native command.
        fs::remove_file(&journal).unwrap();
        fs::create_dir(&journal).unwrap();
        assert_eq!(
            state
                .cancel(cancel.clone(), input, dir, Instant::now())
                .unwrap_err(),
            "client command journal must be a regular file"
        );
        assert_eq!(
            native.command_status(id).unwrap().state,
            CommandState::Pending
        );
        assert_eq!(state.report("cancel-fault", dir).unwrap(), submitted);
        assert_eq!(loader.client_commands().len(), 1);
        assert_eq!(
            fs::read(path(&fixture.0, "cancel-fault")).unwrap(),
            reservation
        );
        // Repair only this owned fixture path, then deliberately request cancellation.
        fs::remove_dir(&journal).unwrap();
        fs::write(&journal, &original).unwrap();
        let accepted = state
            .cancel(cancel.clone(), input, dir, Instant::now())
            .unwrap();
        assert_eq!(accepted["cancel_attempted"], true);
        assert_eq!(accepted["cancel_request_accepted"], true);
        assert_eq!(
            native.command_status(id).unwrap().state,
            CommandState::Cancelled(enshwick::input::CancelReason::ByMod)
        );
        let after_cancel = fs::read(&journal).unwrap();
        assert!(after_cancel.starts_with(&original));
        assert_eq!(
            after_cancel
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty())
                .count(),
            4
        );
        // Rearm only the SDK model: a second native cancel would visibly cancel it again.
        loader.set_command_status(native_id, Status::PENDING, 0);
        let replay = state
            .cancel(cancel.clone(), input, dir, Instant::now())
            .unwrap();
        assert_eq!(replay, accepted);
        assert_eq!(
            native.command_status(id).unwrap().state,
            CommandState::Pending
        );
        assert_eq!(fs::read(&journal).unwrap(), after_cancel);
        assert_eq!(
            fs::read(path(&fixture.0, "cancel-fault")).unwrap(),
            reservation
        );
        assert_eq!(loader.client_commands().len(), 1);
        assert_eq!(
            state.inventory(dir).unwrap()["commands"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let mut reopened = State::default();
        assert_eq!(
            reopened.report("cancel-fault", dir).unwrap()["state"],
            "prior_boot_unavailable"
        );
        assert_eq!(
            reopened
                .cancel(cancel, input, dir, Instant::now())
                .unwrap_err(),
            "client cancellation requires a retained operation; no ID reconstruction"
        );
        assert_eq!(
            native.command_status(id).unwrap().state,
            CommandState::Pending
        );
        assert_eq!(fs::read(&journal).unwrap(), after_cancel);
    }

    #[test]
    fn unusable_native_contexts_refuse_before_reservation_or_dispatch() {
        let loader = enshwick::test::FakeHost::new()
            .side(Side::CLIENT)
            .player_input()
            .install();
        // SAFETY: these are this test module's framework-generated exports.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let input = *INPUT.lock().unwrap();
        input.unwrap().client_snapshot().unwrap();
        let _ = loader.draw_frame();
        let fixture = Fixture::new();
        fs::write(fixture.0.join(write_gate::FLAG), b"").unwrap();
        let dir = Some(fixture.0.as_path());
        let valid = snapshot(8, "InGame", "");
        let mut state = State::default();
        loader.set_client_snapshot(valid);
        assert!(context(input.unwrap()).unwrap().fresh);
        assert_eq!(
            state
                .run(dry("valid-context"), input, dir, Instant::now())
                .unwrap()["state"],
            "dry"
        );
        for context in [
            Raw::EMPTY,
            Raw {
                state: Raw::OFF,
                ..valid
            },
            Raw {
                state: Raw::STALE,
                ..valid
            },
            Raw {
                age_ms: 1001,
                ..valid
            },
            Raw {
                session: 0,
                ..valid
            },
            Raw {
                available: 0,
                ..valid
            },
            Raw {
                hud_screen: Str::new(""),
                ..valid
            },
        ] {
            loader.set_client_snapshot(context);
            for action in [
                gate::Action::Open("Map".into()),
                gate::Action::Close,
                gate::Action::Tab("Map".into()),
                gate::Action::Recipe(73),
                gate::Action::Look(0.1, 0.0),
                gate::Action::Move([0.0, 1.0], 1),
            ] {
                let mut request = real(dry("unusable-context"), &json!({"preview":"old-context"}));
                request.action = action;
                assert_eq!(
                    state.run(request, input, dir, Instant::now()).unwrap_err(),
                    "client command requires a fresh nonzero session and readable HUD screen"
                );
            }
        }
        assert_eq!(state.inventory(dir).unwrap()["commands"], json!([]));
        assert!(loader.client_commands().is_empty());
        assert!(!fixture.0.join("client-commands").exists());
        assert!(!fixture.0.join("journal.jsonl").exists());
    }

    #[test]
    fn changed_context_after_durable_intent_refuses_without_dispatch_or_replay() {
        use enshwick::abi::{HostApi, ModHandle, Status};
        use std::cell::RefCell;
        struct Script {
            calls: usize,
            initial: Raw,
            changed: Raw,
            delay_second: bool,
            revoke_second: bool,
            dir: std::path::PathBuf,
            operation: String,
            intent_seen: bool,
        }
        thread_local! {
            static SCRIPT: RefCell<Option<Script>> = const { RefCell::new(None) };
        }
        extern "system" fn read_snapshot(_me: ModHandle, out: *mut Raw) -> Status {
            if out.is_null() {
                return Status::BAD_ARGS;
            }
            SCRIPT.with(|slot| {
                let mut slot = slot.borrow_mut();
                let script = slot.as_mut().unwrap();
                script.calls += 1;
                let value = if script.calls == 1 {
                    script.initial
                } else {
                    script.intent_seen = path(&script.dir, &script.operation).is_file()
                        && fs::read_to_string(script.dir.join("journal.jsonl"))
                            .is_ok_and(|journal| journal.contains("client_command_intent"));
                    if script.revoke_second {
                        fs::remove_file(script.dir.join(write_gate::FLAG)).unwrap();
                    }
                    if script.delay_second {
                        std::thread::sleep(gate::WAIT + std::time::Duration::from_millis(50));
                    }
                    script.changed
                };
                // SAFETY: the SDK getter supplies a writable, full-size Raw output.
                unsafe { out.write(value) };
                Status::OK
            })
        }
        let loader = enshwick::test::FakeHost::new()
            .side(Side::CLIENT)
            .player_input()
            .install();
        // SAFETY: api() points to this installed fake's complete published table.
        let mut api: HostApi = unsafe { *loader.api() };
        api.client_snapshot = read_snapshot;
        let api = Box::leak(Box::new(api));
        // SAFETY: our generated init receives an immutable process-lifetime fake
        // table and its own fake handle. Only the copied-snapshot getter changes.
        assert_eq!(
            unsafe { enshwick_mod_init(api, loader.handle()) },
            Status::OK
        );
        // SAFETY: this mod's matching init just succeeded on this fake.
        assert_eq!(unsafe { loader.ready(enshwick_mod_ready) }, Status::OK);
        let input = *INPUT.lock().unwrap();
        let initial = snapshot(8, "InGame", "");
        for (name, changed, error, delay_second, revoke_second) in [
            ("valid", initial, None, false, false),
            (
                "stale",
                Raw {
                    state: Raw::STALE,
                    ..initial
                },
                Some("client command requires a fresh nonzero session and readable HUD screen"),
                false,
                false,
            ),
            (
                "session",
                snapshot(9, "InGame", ""),
                Some("client preview changed; inspect current session/screen and preview again"),
                false,
                false,
            ),
            (
                "screen",
                snapshot(8, "Backpack", ""),
                Some("client preview changed; inspect current session/screen and preview again"),
                false,
                false,
            ),
            (
                "late",
                initial,
                Some("client command deadline expired before native submission"),
                true,
                false,
            ),
            (
                "revoked",
                initial,
                Some(
                    "writes are off: create ALLOW_WRITES in the mod's folder (and remove it when done)",
                ),
                false,
                true,
            ),
        ] {
            let fixture = Fixture::new();
            fs::write(fixture.0.join(write_gate::FLAG), b"").unwrap();
            let operation = format!("recheck-{name}");
            let mut state = State::default();
            let request = dry(&operation);
            let preview = gate::preview(
                &state.boot,
                &request,
                &gate::Context {
                    fresh: true,
                    age_ms: Some(0),
                    session: 8,
                    screens_read: true,
                    screen: Some("InGame".into()),
                    tab_read: true,
                    tab: None,
                },
            );
            SCRIPT.with(|slot| {
                *slot.borrow_mut() = Some(Script {
                    calls: 0,
                    initial,
                    changed,
                    delay_second,
                    revoke_second,
                    dir: fixture.0.clone(),
                    operation: operation.clone(),
                    intent_seen: false,
                })
            });
            let request = real(request, &json!({"preview":preview}));
            let dir = Some(fixture.0.as_path());
            let before = loader.client_commands().len();
            let result = state
                .run(request.clone(), input, dir, Instant::now())
                .unwrap();
            SCRIPT.with(|slot| {
                let slot = slot.borrow();
                let script = slot.as_ref().unwrap();
                assert_eq!(script.calls, 2, "{name}: {result}");
                assert!(script.intent_seen, "second read must follow durable intent");
            });
            assert_eq!(
                result["state"],
                if error.is_some() {
                    "refused_or_unknown"
                } else {
                    "queued"
                },
                "{result}"
            );
            assert_eq!(
                result["native_submitted"],
                error.is_none(),
                "{name}: {result}"
            );
            assert_eq!(
                loader.client_commands().len(),
                before + usize::from(error.is_none())
            );
            if let Some(error) = error {
                assert_eq!(result["error"], error);
                if revoke_second {
                    assert!(!fixture.0.join(write_gate::FLAG).exists());
                    fs::write(fixture.0.join(write_gate::FLAG), b"").unwrap();
                }
                let reservation = fs::read(path(&fixture.0, &operation)).unwrap();
                let journal = fs::read(fixture.0.join("journal.jsonl")).unwrap();
                assert_eq!(
                    retained_outcome(
                        state
                            .run(request.clone(), input, dir, Instant::now())
                            .unwrap()
                    ),
                    retained_outcome(result)
                );
                assert_eq!(loader.client_commands().len(), before);
                assert_eq!(fs::read(path(&fixture.0, &operation)).unwrap(), reservation);
                assert_eq!(fs::read(fixture.0.join("journal.jsonl")).unwrap(), journal);
                assert!(
                    State::default()
                        .run(request, input, dir, Instant::now())
                        .unwrap_err()
                        .contains("prior boot")
                );
                assert_eq!(fs::read(fixture.0.join("journal.jsonl")).unwrap(), journal);
            }
        }
    }

    #[test]
    fn retained_replies_expose_poll_age_without_refreshing_or_redispatching() {
        let loader = enshwick::test::FakeHost::new()
            .side(Side::CLIENT)
            .player_input()
            .install();
        // SAFETY: these are this test module's framework-generated exports.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let input = *INPUT.lock().unwrap();
        loader.set_client_snapshot(snapshot(9, "InGame", ""));
        let fixture = Fixture::new();
        fs::write(fixture.0.join(write_gate::FLAG), b"").unwrap();
        let dir = Some(fixture.0.as_path());
        let mut state = State::default();
        let preview = state
            .run(dry("reply-age"), input, dir, Instant::now())
            .unwrap();
        let request = real(dry("reply-age"), &preview);
        let queued = state
            .run(request.clone(), input, dir, Instant::now())
            .unwrap();
        assert_eq!(queued["status_observation"]["stale"], true);
        assert!(queued["status_observation"]["age_ms"].is_null());
        assert_eq!(queued["status_observation"]["polling_finished"], false);
        state.observe(input, dir);
        state.operations.get_mut("reply-age").unwrap().observed_at =
            Some(Instant::now() - std::time::Duration::from_secs(2));
        let journal = fs::read(fixture.0.join("journal.jsonl")).unwrap();
        let replay = state
            .run(dry("reply-age"), input, dir, Instant::now())
            .unwrap();
        assert_eq!(replay["native_state"], "pending");
        assert_eq!(replay["status_observation"]["stale"], true);
        assert!(replay["status_observation"]["age_ms"].as_u64().unwrap() >= 2000);
        let cancel_dry =
            gate::cancel(json!({"operation":"reply-age"}).as_object().unwrap()).unwrap();
        let cancel_preview = state
            .cancel(cancel_dry, input, dir, Instant::now())
            .unwrap();
        assert_eq!(
            cancel_preview["last_outcome"]["status_observation"]["stale"],
            true
        );
        assert_eq!(fs::read(fixture.0.join("journal.jsonl")).unwrap(), journal);
        let cancel = gate::cancel(json!({"operation":"reply-age","dry":false,"confirm":write_gate::CONFIRM,"preview":cancel_preview["preview"]}).as_object().unwrap()).unwrap();
        let submitted = state
            .cancel(cancel.clone(), input, dir, Instant::now())
            .unwrap();
        assert_eq!(submitted["cancel_attempted"], true);
        assert_eq!(submitted["status_observation"]["stale"], true);
        let journal = fs::read(fixture.0.join("journal.jsonl")).unwrap();
        let again = state
            .cancel(cancel.clone(), input, dir, Instant::now())
            .unwrap();
        assert_eq!(again["status_observation"]["stale"], true);
        assert_eq!(fs::read(fixture.0.join("journal.jsonl")).unwrap(), journal);
        state.observe(input, dir);
        state.operations.get_mut("reply-age").unwrap().observed_at =
            Some(Instant::now() - std::time::Duration::from_secs(2));
        let journal = fs::read(fixture.0.join("journal.jsonl")).unwrap();
        for reply in [
            state.run(request, input, dir, Instant::now()).unwrap(),
            state.cancel(cancel, input, dir, Instant::now()).unwrap(),
        ] {
            assert_eq!(reply["native_state"], "cancelled");
            assert_eq!(reply["status_observation"]["polling_finished"], true);
            assert_eq!(reply["status_observation"]["stale"], true);
            assert_eq!(
                reply["status_observation"]["current_context_verified"],
                false
            );
        }
        assert_eq!(loader.client_commands().len(), 1);
        assert_eq!(fs::read(fixture.0.join("journal.jsonl")).unwrap(), journal);
    }

    #[test]
    fn status_distinguishes_never_polled_fresh_stalled_and_historical_terminal_results() {
        let loader = enshwick::test::FakeHost::new()
            .side(Side::CLIENT)
            .player_input()
            .install();
        // SAFETY: these are this test module's framework-generated exports.
        assert_eq!(
            unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
            enshwick::abi::Status::OK
        );
        let input = *INPUT.lock().unwrap();
        loader.set_client_snapshot(snapshot(9, "InGame", ""));
        let fixture = Fixture::new();
        fs::write(fixture.0.join(write_gate::FLAG), b"").unwrap();
        let dir = Some(fixture.0.as_path());
        let mut state = State::default();
        let preview = state
            .run(dry("freshness"), input, dir, Instant::now())
            .unwrap();
        state
            .run(real(dry("freshness"), &preview), input, dir, Instant::now())
            .unwrap();
        let never = state.report("freshness", dir).unwrap();
        assert!(never["status_observation"]["age_ms"].is_null());
        assert_eq!(never["status_observation"]["stale"], true);
        state.observe(input, dir);
        let fresh = state.report("freshness", dir).unwrap();
        assert_eq!(
            fresh["status_observation"]["stale"],
            fresh["status_observation"]["age_ms"].as_u64().unwrap() > 1000
        );
        assert_eq!(fresh["status_observation"]["polling_finished"], false);
        state.operations.get_mut("freshness").unwrap().observed_at =
            Some(Instant::now() - std::time::Duration::from_secs(2));
        let journal = fs::read(fixture.0.join("journal.jsonl")).unwrap();
        let stale = state.report("freshness", dir).unwrap();
        assert_eq!(stale["native_state"], "pending");
        assert_eq!(stale["status_observation"]["stale"], true);
        assert!(stale["status_observation"]["age_ms"].as_u64().unwrap() >= 2000);
        assert_eq!(
            state.inventory(dir).unwrap()["commands"][0]["status_observation"]["stale"],
            true
        );
        assert_eq!(fs::read(fixture.0.join("journal.jsonl")).unwrap(), journal);
        let id = loader.client_commands()[0].0;
        loader.set_command_status(id, Status::OBSERVED, 0);
        state.observe(input, dir);
        state.operations.get_mut("freshness").unwrap().observed_at =
            Some(Instant::now() - std::time::Duration::from_secs(2));
        let done = state.report("freshness", dir).unwrap();
        assert_eq!(done["native_state"], "observed");
        assert_eq!(done["status_observation"]["polling_finished"], true);
        assert_eq!(done["status_observation"]["stale"], true);
        assert_eq!(done["rendered_verified"], false);
        let terminal_journal = fs::read(fixture.0.join("journal.jsonl")).unwrap();
        state.observe(input, dir);
        assert_eq!(
            state.report("freshness", dir).unwrap()["status_observation"]["stale"],
            true
        );
        assert_eq!(
            fs::read(fixture.0.join("journal.jsonl")).unwrap(),
            terminal_journal
        );
        let prior = State::default().report("freshness", dir).unwrap();
        assert!(prior["status_observation"]["age_ms"].is_null());
        assert!(prior["status_observation"]["polling_finished"].is_null());
        assert_eq!(prior["status_observation"]["stale"], true);
        assert_eq!(loader.client_commands().len(), 1);
        let preview = state
            .run(dry("poll-error"), input, dir, Instant::now())
            .unwrap();
        state
            .run(
                real(dry("poll-error"), &preview),
                input,
                dir,
                Instant::now(),
            )
            .unwrap();
        let id = loader.client_commands()[1].0;
        loader.set_command_status(id, Status::APPLIED, 0);
        state.observe(input, dir);
        assert_eq!(
            state.report("poll-error", dir).unwrap()["native_state"],
            "applied"
        );
        assert_eq!(state.report("poll-error", dir).unwrap()["native_age_ms"], 0);
        // Model an SDK identity no longer available in its native status store.
        state.operations.get_mut("poll-error").unwrap().id = Some(CommandId::new(999));
        state.observe(input, dir);
        let unknown = state.report("poll-error", dir).unwrap();
        assert_eq!(unknown["native_state"], "unknown");
        assert!(unknown["native_error"].is_string());
        assert!(unknown["native_age_ms"].is_null());
        assert!(unknown["status_observation"]["age_ms"].is_number());
        assert_eq!(unknown["status_observation"]["polling_finished"], true);
        assert_eq!(
            unknown["status_observation"]["current_context_verified"],
            false
        );
        assert_eq!(loader.client_commands().len(), 2);
        let journal = fs::read(fixture.0.join("journal.jsonl")).unwrap();
        state.observe(input, dir);
        let inventory = state.inventory(dir).unwrap();
        let row = inventory["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["operation"] == "poll-error")
            .unwrap();
        assert!(row["native_age_ms"].is_null());
        assert_eq!(row["native_state"], "unknown");
        let replay = state
            .run(
                real(dry("poll-error"), &preview),
                input,
                dir,
                Instant::now(),
            )
            .unwrap();
        assert_eq!(retained_outcome(replay), retained_outcome(unknown));
        assert_eq!(fs::read(fixture.0.join("journal.jsonl")).unwrap(), journal);
        assert_eq!(loader.client_commands().len(), 2);
    }
}
