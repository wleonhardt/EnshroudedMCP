//! Inactive, file-driven native qualifier. No MCP registration or World access.
use enshwick::{
    Load, Mod, Off,
    input::{
        ClientCommand, CommandAction, CommandId, CommandState, Input, JoinPhase, SnapshotState,
    },
};
use policy::client_connection::{self as admission, Action, Context, Profile, Screen, Spec};
use serde_json::{Value, json};
use std::sync::Arc;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const REVISION: &str = "33cf4dc149f88d1abe3d3c11a12053d0ab94967f";
use admission::read_object as object;

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

struct Job {
    spec: Spec,
    id: Option<CommandId>,
    core: Value,
    pending: bool,
    joined: bool,
    joined_session: Option<u64>,
    left: bool,
}
#[cfg(test)]
type BeforeFinal = Box<dyn FnOnce(&mut State) + Send>;
struct State {
    dir: PathBuf,
    profile: Profile,
    boot: String,
    journal: File,
    header: Value,
    started: Instant,
    heartbeat: Option<Instant>,
    previous_screen: Option<Screen>,
    frame_at: Option<Instant>,
    sequence: u64,
    next: u32,
    jobs: Vec<Job>,
    last_reply: Value,
    halted: bool,
    #[cfg(test)]
    before_final: Option<BeforeFinal>,
}
impl State {
    fn new(
        dir: PathBuf,
        profile: Profile,
        boot: String,
        header: enshwick::abi::Header,
    ) -> Result<Self, String> {
        admission::provider(
            header.abi_major.into(),
            header.abi_minor.into(),
            header.size,
            std::mem::size_of::<enshwick::abi::HostApi>() as u32,
        )?;
        admission::start_session(&dir, &boot)?;
        fs::create_dir(dir.join("operations"))
            .map_err(|_| "fresh operation directory unavailable")?;
        let journal = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("observations.jsonl"))
            .map_err(|_| "fresh observation journal unavailable")?;
        Ok(Self {
            dir,
            profile,
            boot,
            journal,
            header: json!({"abi_major":header.abi_major,"abi_minor":header.abi_minor,"table_bytes":header.size}),
            started: Instant::now(),
            heartbeat: None,
            previous_screen: None,
            frame_at: None,
            sequence: 0,
            next: 1,
            jobs: Vec::new(),
            last_reply: Value::Null,
            halted: false,
            #[cfg(test)]
            before_final: None,
        })
    }
    fn record(&mut self, value: Value) -> Result<(), String> {
        let result = self.append(value);
        if result.is_err() {
            self.halted = true;
        }
        result
    }
    fn append(&mut self, value: Value) -> Result<(), String> {
        let mut bytes =
            serde_json::to_vec(&value).map_err(|_| "observation serialization unavailable")?;
        bytes.push(b'\n');
        let length = self
            .journal
            .metadata()
            .map_err(|_| "observation budget unavailable")?
            .len();
        policy::write_gate::journal_budget(length, bytes.len(), 0)?;
        self.journal
            .write_all(&bytes)
            .and_then(|()| self.journal.sync_all())
            .map_err(|_| "observation unconfirmed; no retry".into())
    }
    fn poll(&mut self, input: Input) -> Result<(), String> {
        let (context, _) = self.context(self.previous_screen.unwrap_or(Screen::Other), input);
        let mut changes = Vec::new();
        for job in &mut self.jobs {
            let Some(id) = job.id else { continue };
            let (core, age, pending, joined, left) = match input.command_status(id) {
                Ok(status) => (
                    json!({"native_state":format!("{:?}",status.state),"native_phase":status.phase.map(|p|format!("{p:?}")),"status_error":null}),
                    Some(status.age_ms),
                    matches!(status.state, CommandState::Pending | CommandState::Applied)
                        && status.phase != Some(JoinPhase::InWorld),
                    matches!(status.state, CommandState::Observed)
                        && status.phase == Some(JoinPhase::InWorld),
                    matches!(status.state, CommandState::Observed)
                        && status.phase == Some(JoinPhase::Left),
                ),
                Err(error) => (
                    json!({"native_state":null,"native_phase":null,"status_error":format!("{error:?}")}),
                    None,
                    false,
                    false,
                    false,
                ),
            };
            job.joined |= joined;
            job.left |= left;
            job.pending = pending && !job.joined && !job.left;
            let bound = admission::bind_join_session(job.joined_session, joined, &context);
            if bound != job.joined_session {
                job.joined_session = bound;
                changes.push(json!({"kind":"native_session_binding","unix_ms":now_ms(),"operation":job.spec.operation,
                    "session":bound,"ui_sequence":self.sequence,"observation_non_atomic":true,"peer_authenticated":false}));
            }
            if core != job.core {
                job.core = core.clone();
                changes.push(json!({"kind":"native_status","unix_ms":now_ms(),"operation":job.spec.operation,"native":core,"age_ms":age,"workflow_accepted":false}));
            }
        }
        for value in changes {
            self.record(value)?;
        }
        Ok(())
    }
    fn context(&self, screen: Screen, input: Input) -> (Context, Value) {
        self.context_jobs(screen, input, &self.jobs)
    }
    fn context_jobs(&self, screen: Screen, input: Input, jobs: &[Job]) -> (Context, Value) {
        let (session, fresh, age, peer, metadata) = match input.client_snapshot() {
            Ok(copy) => {
                let age = copy.age_ms();
                let peer = self.profile.matches_peer(copy.server());
                (
                    Some(copy.session()),
                    copy.state() == SnapshotState::Fresh,
                    age,
                    peer,
                    json!({"state":format!("{:?}",copy.state()),"session":copy.session(),"age_ms":age,"peer_matches":peer,"read_error":null}),
                )
            }
            Err(error) => (
                None,
                false,
                None,
                None,
                json!({"state":null,"session":null,"age_ms":null,"peer_matches":null,"read_error":format!("{error:?}")}),
            ),
        };
        let owned = admission::ownership(
            screen,
            jobs.iter().map(|job| admission::OperationObservation {
                operation: &job.spec.operation,
                action: &job.spec.action,
                pending: job.pending,
                joined_session: job.joined_session,
                left: job.left,
            }),
        );
        (
            Context {
                screen,
                age_ms: self
                    .frame_at
                    .and_then(|at| u64::try_from(at.elapsed().as_millis()).ok()),
                sequence: self.sequence,
                native_session: session,
                native_fresh: fresh,
                native_age_ms: age,
                peer_matches: peer,
                owned,
            },
            metadata,
        )
    }
    fn execute(
        &mut self,
        spec: Spec,
        screen: Screen,
        input: Input,
        received: Instant,
        file_age: Duration,
    ) -> Result<Value, String> {
        if self.halted {
            return Err("qualification halted; no native retry".into());
        }
        admission::lease(self.started.elapsed())?;
        let (context, _) = self.context(screen, input);
        let flag = policy::write_gate::flag_age(&self.dir.join(policy::write_gate::FLAG));
        admission::authorize(
            &self.boot,
            &spec,
            &context,
            &self.profile.label,
            flag,
            file_age.saturating_add(received.elapsed()),
        )?;
        admission::new_operation(
            &spec.operation,
            self.jobs.iter().map(|job| job.spec.operation.as_str()),
        )?;
        let preview = admission::preview(&self.boot, &spec, &context);
        if spec.dry {
            return Ok(
                json!({"state":"dry","preview":preview,"request":admission::payload(&spec.action),"native_submitted":false}),
            );
        }
        if let Err(error) =
            admission::reserve(&self.dir.join("operations"), &self.boot, &spec, &context)
        {
            self.halted = true;
            return Err(error);
        }
        // Keep the identity even if an authority/deadline change refuses dispatch.
        self.jobs.push(Job {
            spec: spec.clone(),
            id: None,
            core: Value::Null,
            pending: false,
            joined: false,
            joined_session: None,
            left: false,
        });
        #[cfg(test)]
        if let Some(before_final) = self.before_final.take() {
            before_final(self);
        }
        self.poll(input)?;
        let (final_context, _) = self.context_before(&spec, screen, input);
        let final_flag = policy::write_gate::flag_age(&self.dir.join(policy::write_gate::FLAG));
        admission::lease(self.started.elapsed())?;
        admission::authorize(
            &self.boot,
            &spec,
            &final_context,
            &self.profile.label,
            final_flag,
            file_age.saturating_add(received.elapsed()),
        )?;
        let submitted = match &spec.action {
            Action::Join(_) => input
                .client_command(&ClientCommand::new(
                    CommandAction::Join {
                        server: self.profile.query(),
                        password: None,
                    },
                    0,
                ))
                .map(Some),
            Action::Leave => input
                .client_command(&ClientCommand::new(
                    CommandAction::Leave,
                    final_context.native_session.unwrap(),
                ))
                .map(Some),
            Action::Cancel(target) => {
                let id = self
                    .jobs
                    .iter()
                    .find(|job| job.spec.operation == *target)
                    .and_then(|job| job.id)
                    .ok_or("retained native target unavailable")?;
                input.cancel_command(id).map(|()| None)
            }
        };
        let reply = match submitted {
            Ok(id) => {
                self.jobs.last_mut().unwrap().id = id;
                json!({"state":"submitted","native_id_retained":id.is_some(),"cancellation_completed":false})
            }
            Err(error) => {
                json!({"state":"native_error","reason":format!("{error:?}"),"automatic_retry":false})
            }
        };
        self.record(json!({"kind":"native_submission","unix_ms":now_ms(),"operation":spec.operation,"request":admission::payload(&spec.action),"receipt":reply}))?;
        Ok(reply)
    }
    fn context_before(&self, spec: &Spec, screen: Screen, input: Input) -> (Context, Value) {
        // The just-reserved entry has no SDK identity. Derive the prior context
        // from only entries that preceded it; intent is not execution evidence.
        assert_eq!(
            self.jobs.last().map(|job| job.spec.operation.as_str()),
            Some(spec.operation.as_str())
        );
        self.context_jobs(screen, input, &self.jobs[..self.jobs.len() - 1])
    }
    fn frame(&mut self, screen: Screen, input: Input) -> Result<(), String> {
        if self.halted {
            return Ok(());
        }
        self.frame_at = Some(Instant::now());
        if admission::lease(self.started.elapsed()).is_err() {
            self.record(
                json!({"kind":"lease_expired","unix_ms":now_ms(),"native_cleanup_attempted":false}),
            )?;
            self.halted = true;
            return Ok(());
        }
        if self.previous_screen != Some(screen) {
            self.sequence = self.sequence.saturating_add(1);
            self.previous_screen = Some(screen);
        }
        self.poll(input)?;
        if self.next <= admission::MAX_REQUESTS {
            let path = self.dir.join(format!("request-{:04}.json", self.next));
            if let Some((value, age)) = object(&path)? {
                let request = self.next;
                self.next += 1;
                let received = Instant::now();
                let result = value
                    .as_object()
                    .ok_or("request must be an object")
                    .map_err(str::to_owned)
                    .and_then(admission::check)
                    .and_then(|spec| self.execute(spec, screen, input, received, age));
                if self.halted {
                    return Err(
                        "qualification evidence unconfirmed; no further recording or native retry"
                            .into(),
                    );
                }
                self.last_reply = match result {
                    Ok(value) => json!({"request_index":request,"result":value}),
                    Err(error) => json!({"request_index":request,"error":error}),
                };
                self.record(
                    json!({"kind":"request_reply","unix_ms":now_ms(),"reply":self.last_reply}),
                )?;
            }
        }
        if self
            .heartbeat
            .is_none_or(|last| last.elapsed() >= Duration::from_millis(200))
        {
            let (context, native) = self.context(screen, input);
            self.record(json!({"kind":"probe_observation","unix_ms":now_ms(),"compiled_sdk_revision":REVISION,"compiled_abi":"12.16","provider_header":self.header,
                "profile":self.profile.label,"ui_screen":format!("{screen:?}"),"ui_age_ms":context.age_ms,"ui_sequence":self.sequence,"owned":format!("{:?}",context.owned),"native_snapshot":native,
                "next_request":self.next,"session_lease_ms":self.started.elapsed().as_millis(),"provider_fidelity_verified":false,"independent_roster_verified":false,"character_verified":false,"rendered_verified":false,"workflow_accepted":false}))?;
            self.heartbeat = Some(Instant::now());
        }
        Ok(())
    }
}

struct LifecycleProbe;
impl Mod for LifecycleProbe {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        // The framework metadata supports server/both. The paired server loads
        // this identity passively; execution and profile access are client-only.
        if load.host().build().side != enshwick::abi::Side::CLIENT {
            return Ok(Self);
        }
        let dir = load.mod_dir()?;
        let value = object(&dir.join("profile.json"))
            .map_err(Off::new)?
            .ok_or_else(|| Off::new("explicit private qualification profile required"))?
            .0;
        let profile = admission::profile(&value).map_err(Off::new)?;
        let boot = format!("probe-{}-{}", std::process::id(), now_ms());
        let state = State::new(dir, profile, boot, load.host().api().header).map_err(Off::new)?;
        let state = Arc::new(Mutex::new(state));
        let input = load.input()?;
        load.on_frame(move |frame| {
            let screen = match frame.screen() {
                enshwick::ui::Screen::Menu => Screen::Menu,
                enshwick::ui::Screen::InGame => Screen::InGame,
                _ => Screen::Other,
            };
            if let Ok(mut state) = state.lock()
                && state.frame(screen, input).is_err()
            {
                state.halted = true;
            }
        })?;
        Ok(Self)
    }
}
#[cfg(not(test))]
enshwick::declare_mod!(LifecycleProbe);

#[cfg(test)]
mod tests;
