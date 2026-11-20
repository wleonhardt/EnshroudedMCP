use super::*;
use enshwick::abi::{
    ClientCommandStatus as Raw, ClientSnapshot as Snapshot, Header, Side, Status, Str,
};
use std::path::Path;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::atomic::{AtomicU64, Ordering},
};

static INPUT: Mutex<Option<Input>> = Mutex::new(None);
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Driver;
impl Mod for Driver {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        if load.host().build().side == Side::SERVER {
            LifecycleProbe::init(load)?;
            return Ok(Self);
        }
        *INPUT.lock().unwrap() = Some(load.input()?);
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Driver, "id = \"test.lifecycle-control\"\nside = \"both\"");

fn setup() -> (enshwick::test::Installed, Input, State, String) {
    let loader = enshwick::test::FakeHost::new()
        .side(Side::CLIENT)
        .player_input()
        .install();
    // SAFETY: exports belong to this isolated fake-only driver.
    assert_eq!(
        unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    let input = INPUT.lock().unwrap().unwrap();
    let _ = loader.draw_frame();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap();
    let parent = root.join("gamedata/qualification-tests/lifecycle-control");
    fs::create_dir_all(&parent).unwrap();
    assert!(parent.canonicalize().unwrap().starts_with(&root));
    let dir = parent.join(format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, 7)).to_string();
    let profile = admission::profile(&json!({"profile":"isolated-test","query":address})).unwrap();
    let mut state = State::new(
        dir,
        profile,
        "boot-1".into(),
        Header::of::<enshwick::abi::HostApi>(),
    )
    .unwrap();
    // The published fake draws an in-game frame. Menu is a synthetic argument
    // to the same state handler; no rendered or native Menu claim follows.
    state.frame(Screen::Menu, input).unwrap();
    (loader, input, state, address)
}
fn clean(state: State) {
    let dir = state.dir.clone();
    drop(state);
    assert!(
        dir.canonicalize().unwrap().starts_with(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .canonicalize()
                .unwrap()
        )
    );
    for entry in fs::read_dir(dir.join("operations")).unwrap() {
        fs::remove_file(entry.unwrap().path()).unwrap();
    }
    fs::remove_dir(dir.join("operations")).unwrap();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        assert!(path.is_file());
        fs::remove_file(path).unwrap();
    }
    fs::remove_dir(dir).unwrap();
}
fn dry(operation: &str, action: &str, target: Option<&str>) -> Spec {
    let mut value = json!({"operation":operation,"action":action});
    if action == "join" {
        value["profile"] = json!("isolated-test");
    }
    if let Some(target) = target {
        value["target_operation"] = json!(target);
    }
    admission::check(value.as_object().unwrap()).unwrap()
}
fn real(state: &mut State, input: Input, screen: Screen, mut spec: Spec) -> Spec {
    let reply = state
        .execute(spec.clone(), screen, input, Instant::now(), Duration::ZERO)
        .unwrap();
    assert_eq!(reply["state"], "dry");
    spec.dry = false;
    spec.preview = Some(reply["preview"].as_str().unwrap().into());
    spec.confirm = Some("server-write".into());
    fs::write(state.dir.join(policy::write_gate::FLAG), []).unwrap();
    spec
}
fn submit_join(state: &mut State, input: Input) {
    let spec = real(state, input, Screen::Menu, dry("join-001", "join", None));
    assert_eq!(
        state
            .execute(spec, Screen::Menu, input, Instant::now(), Duration::ZERO)
            .unwrap()["state"],
        "submitted"
    );
    state.poll(input).unwrap();
}

#[test]
fn file_dry_then_real_submits_once_and_redacts_private_target() {
    let (loader, input, mut state, address) = setup();
    let request = json!({"operation":"join-001","action":"join","profile":"isolated-test"});
    fs::write(state.dir.join("request-0001.json"), request.to_string()).unwrap();
    state.frame(Screen::Menu, input).unwrap();
    assert_eq!(state.last_reply["result"]["state"], "dry");
    assert!(loader.client_commands().is_empty());
    assert_eq!(
        fs::read_dir(state.dir.join("operations")).unwrap().count(),
        0
    );
    let mut request = request;
    request["dry"] = json!(false);
    request["preview"] = state.last_reply["result"]["preview"].clone();
    request["confirm"] = json!("server-write");
    fs::write(state.dir.join(policy::write_gate::FLAG), []).unwrap();
    fs::write(state.dir.join("request-0002.json"), request.to_string()).unwrap();
    state.frame(Screen::Menu, input).unwrap();
    assert_eq!(state.last_reply["result"]["state"], "submitted");
    assert_eq!(loader.client_commands().len(), 1);
    fs::write(state.dir.join("request-0003.json"), request.to_string()).unwrap();
    state.frame(Screen::Menu, input).unwrap();
    assert!(state.last_reply["error"].is_string());
    assert_eq!(loader.client_commands().len(), 1);
    let journal = fs::read_to_string(state.dir.join("observations.jsonl")).unwrap();
    assert!(!journal.contains(&address));
    assert!(!journal.contains("synthetic-do-not-disclose"));
    let intent = fs::read_to_string(state.dir.join("operations/connection-join-001.json")).unwrap();
    assert!(intent.contains("connection_intent"));
    assert!(!intent.contains(&address));
    clean(state);
}

#[test]
fn retained_pending_join_can_be_cancelled_once_without_replacement_join() {
    let (loader, input, mut state, _) = setup();
    submit_join(&mut state, input);
    let spec = real(
        &mut state,
        input,
        Screen::Menu,
        dry("cancel-001", "cancel", Some("join-001")),
    );
    let receipt = state
        .execute(
            spec.clone(),
            Screen::Menu,
            input,
            Instant::now(),
            Duration::ZERO,
        )
        .unwrap();
    assert_eq!(receipt["state"], "submitted");
    assert_eq!(receipt["cancellation_completed"], false);
    state.poll(input).unwrap();
    assert_eq!(loader.client_commands().len(), 1);
    assert!(matches!(
        input
            .command_status(state.jobs[0].id.unwrap())
            .unwrap()
            .state,
        CommandState::Cancelled(_)
    ));
    assert!(
        state
            .execute(spec, Screen::Menu, input, Instant::now(), Duration::ZERO)
            .is_err()
    );
    assert_eq!(loader.client_commands().len(), 1);
    clean(state);
}

#[test]
fn leave_requires_matching_fresh_join_then_blocks_overlapping_leave() {
    let (loader, input, mut state, address) = setup();
    submit_join(&mut state, input);
    let id = loader.client_commands()[0].0;
    loader.set_command_status(id, Raw::OBSERVED, 0);
    loader.set_command_phase(id, Raw::PHASE_IN_WORLD, 0);
    loader.set_client_snapshot(Snapshot {
        state: Snapshot::FRESH,
        session: 9,
        age_ms: u64::MAX,
        server: Str::new(&address),
        ..Snapshot::EMPTY
    });
    state.frame(Screen::InGame, input).unwrap();
    assert!(
        state
            .execute(
                dry("leave-001", "leave", None),
                Screen::InGame,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err()
    );
    loader.set_client_snapshot(Snapshot {
        state: Snapshot::FRESH,
        session: 9,
        age_ms: 0,
        server: Str::new(&address),
        ..Snapshot::EMPTY
    });
    state.frame(Screen::InGame, input).unwrap();
    let spec = real(
        &mut state,
        input,
        Screen::InGame,
        dry("leave-001", "leave", None),
    );
    assert_eq!(
        state
            .execute(
                spec.clone(),
                Screen::InGame,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .unwrap()["state"],
        "submitted"
    );
    state.poll(input).unwrap();
    assert_eq!(loader.client_commands().len(), 2);
    assert!(
        state
            .execute(spec, Screen::InGame, input, Instant::now(), Duration::ZERO)
            .is_err()
    );
    assert_eq!(loader.client_commands().len(), 2);
    clean(state);
}

#[test]
fn revoked_flag_after_durable_intent_never_reaches_sdk_and_burns_identity() {
    let (loader, input, mut state, _) = setup();
    let spec = real(
        &mut state,
        input,
        Screen::Menu,
        dry("join-001", "join", None),
    );
    let flag = state.dir.join(policy::write_gate::FLAG);
    state.before_final = Some(Box::new(move |_| fs::remove_file(flag).unwrap()));
    assert!(
        state
            .execute(spec, Screen::Menu, input, Instant::now(), Duration::ZERO)
            .is_err()
    );
    assert!(
        state
            .dir
            .join("operations/connection-join-001.json")
            .is_file()
    );
    assert!(loader.client_commands().is_empty());
    assert!(
        state
            .execute(
                dry("join-002", "join", None),
                Screen::Menu,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err()
    );
    assert!(loader.client_commands().is_empty());
    clean(state);
}

#[test]
fn reservation_failure_halts_all_further_dispatch_and_restart_refuses() {
    let (loader, input, mut state, address) = setup();
    let spec = real(
        &mut state,
        input,
        Screen::Menu,
        dry("join-001", "join", None),
    );
    fs::write(state.dir.join("operations/foreign"), b"keep").unwrap();
    assert!(
        state
            .execute(spec, Screen::Menu, input, Instant::now(), Duration::ZERO)
            .is_err()
    );
    assert!(state.halted);
    assert!(
        state
            .execute(
                dry("join-002", "join", None),
                Screen::Menu,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err()
    );
    assert!(loader.client_commands().is_empty());
    let profile = admission::profile(&json!({"profile":"isolated-test","query":address})).unwrap();
    assert!(
        State::new(
            state.dir.clone(),
            profile,
            "boot-2".into(),
            Header::of::<enshwick::abi::HostApi>()
        )
        .is_err()
    );
    clean(state);
}

#[test]
fn old_request_and_expired_lease_do_not_dispatch_or_cancel_implicitly() {
    let (loader, input, mut state, _) = setup();
    let spec = real(
        &mut state,
        input,
        Screen::Menu,
        dry("join-001", "join", None),
    );
    assert!(
        state
            .execute(
                spec,
                Screen::Menu,
                input,
                Instant::now(),
                Duration::from_secs(2)
            )
            .is_err()
    );
    assert!(loader.client_commands().is_empty());
    assert_eq!(
        fs::read_dir(state.dir.join("operations")).unwrap().count(),
        0
    );
    submit_join(&mut state, input);
    let id = state.jobs[0].id.unwrap();
    state.started = Instant::now() - Duration::from_secs(181);
    state.frame(Screen::Menu, input).unwrap();
    assert!(state.halted);
    assert!(matches!(
        input.command_status(id).unwrap().state,
        CommandState::Pending | CommandState::Applied
    ));
    assert_eq!(loader.client_commands().len(), 1);
    clean(state);
}

#[test]
fn unconfirmed_post_submission_recording_never_resubmits() {
    let (loader, input, mut state, _) = setup();
    let spec = real(
        &mut state,
        input,
        Screen::Menu,
        dry("join-001", "join", None),
    );
    state.journal = File::open(state.dir.join("observations.jsonl")).unwrap();
    assert!(
        state
            .execute(
                spec.clone(),
                Screen::Menu,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err()
    );
    assert!(state.halted);
    assert_eq!(loader.client_commands().len(), 1);
    assert!(
        state
            .execute(spec, Screen::Menu, input, Instant::now(), Duration::ZERO)
            .is_err()
    );
    assert_eq!(loader.client_commands().len(), 1);
    clean(state);
}

#[test]
fn native_failure_stays_a_failure_and_never_starts_another_join() {
    let (loader, input, mut state, _) = setup();
    submit_join(&mut state, input);
    let id = loader.client_commands()[0].0;
    loader.set_command_status(id, Raw::FAILED, 0);
    loader.set_command_phase(id, Raw::PHASE_MENU, Raw::WRONG_PASSWORD);
    state.poll(input).unwrap();
    assert!(
        state.jobs[0].core["native_state"]
            .as_str()
            .unwrap()
            .contains("WrongPassword")
    );
    assert!(
        state
            .execute(
                dry("join-002", "join", None),
                Screen::Menu,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err()
    );
    assert_eq!(loader.client_commands().len(), 1);
    clean(state);
}

#[test]
fn paired_server_initialization_is_passive_without_profile_or_input() {
    *INPUT.lock().unwrap() = None;
    let loader = enshwick::test::FakeHost::new().side(Side::SERVER).install();
    // SAFETY: exports belong to this fake driver; it calls the actual server init.
    assert_eq!(
        unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    assert!(INPUT.lock().unwrap().is_none());
    assert!(loader.client_commands().is_empty());
}

#[test]
fn removing_an_intent_file_cannot_reuse_an_in_memory_operation() {
    let (loader, input, mut state, address) = setup();
    submit_join(&mut state, input);
    let join_id = loader.client_commands()[0].0;
    loader.set_command_status(join_id, Raw::OBSERVED, 0);
    loader.set_command_phase(join_id, Raw::PHASE_IN_WORLD, 0);
    loader.set_client_snapshot(Snapshot {
        state: Snapshot::FRESH,
        session: 9,
        age_ms: 0,
        server: Str::new(&address),
        ..Snapshot::EMPTY
    });
    state.frame(Screen::InGame, input).unwrap();
    let leave = real(
        &mut state,
        input,
        Screen::InGame,
        dry("leave-001", "leave", None),
    );
    state
        .execute(leave, Screen::InGame, input, Instant::now(), Duration::ZERO)
        .unwrap();
    let leave_id = loader.client_commands()[1].0;
    loader.set_command_status(leave_id, Raw::OBSERVED, 0);
    loader.set_command_phase(leave_id, Raw::PHASE_LEFT, 0);
    loader.set_client_snapshot(Snapshot::EMPTY);
    state.frame(Screen::Menu, input).unwrap();
    fs::remove_file(state.dir.join("operations/connection-join-001.json")).unwrap();
    let mut repeated = dry("join-001", "join", None);
    assert!(
        state
            .execute(
                repeated.clone(),
                Screen::Menu,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err(),
        "dry preview ignored a retained identity"
    );
    repeated.dry = false;
    repeated.preview = Some(admission::preview(
        &state.boot,
        &repeated,
        &state.context(Screen::Menu, input).0,
    ));
    repeated.confirm = Some("server-write".into());
    assert!(
        state
            .execute(
                repeated,
                Screen::Menu,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err(),
        "memory identity was replayed after file removal"
    );
    assert_eq!(loader.client_commands().len(), 2);
    clean(state);
}

#[test]
fn an_old_join_receipt_cannot_authorize_leave_in_a_different_session() {
    let (loader, input, mut state, address) = setup();
    submit_join(&mut state, input);
    let id = loader.client_commands()[0].0;
    loader.set_command_status(id, Raw::OBSERVED, 0);
    loader.set_command_phase(id, Raw::PHASE_IN_WORLD, 0);
    loader.set_client_snapshot(Snapshot {
        state: Snapshot::FRESH,
        session: 9,
        age_ms: 0,
        server: Str::new(&address),
        ..Snapshot::EMPTY
    });
    state.frame(Screen::InGame, input).unwrap();
    loader.set_client_snapshot(Snapshot {
        state: Snapshot::FRESH,
        session: 10,
        age_ms: 0,
        server: Str::new(&address),
        ..Snapshot::EMPTY
    });
    state.frame(Screen::InGame, input).unwrap();
    assert!(
        state
            .execute(
                dry("leave-001", "leave", None),
                Screen::InGame,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err(),
        "old Join receipt authorized another session"
    );
    assert_eq!(loader.client_commands().len(), 1);
    clean(state);
}

#[test]
fn terminal_join_observation_survives_later_unavailable_status_without_hiding_it() {
    let (loader, input, mut state, address) = setup();
    submit_join(&mut state, input);
    let id = loader.client_commands()[0].0;
    loader.set_command_status(id, Raw::OBSERVED, 0);
    loader.set_command_phase(id, Raw::PHASE_IN_WORLD, 0);
    loader.set_client_snapshot(Snapshot {
        state: Snapshot::FRESH,
        session: 9,
        age_ms: 0,
        server: Str::new(&address),
        ..Snapshot::EMPTY
    });
    state.frame(Screen::InGame, input).unwrap();
    loader.set_command_status(id, Raw::UNAVAILABLE, 0);
    state.frame(Screen::InGame, input).unwrap();
    assert_eq!(state.jobs[0].core["native_state"], "Unavailable");
    assert_eq!(state.jobs[0].joined_session, Some(9));
    assert!(
        state
            .execute(
                dry("leave-001", "leave", None),
                Screen::InGame,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_ok()
    );
    let journal = fs::read_to_string(state.dir.join("observations.jsonl")).unwrap();
    assert!(journal.contains("native_session_binding"));
    assert!(journal.contains("Unavailable"));
    assert_eq!(loader.client_commands().len(), 1);
    clean(state);
}

#[test]
fn copied_frame_that_ages_after_intent_is_refused_before_native_submission() {
    let (loader, input, mut state, _) = setup();
    let spec = real(
        &mut state,
        input,
        Screen::Menu,
        dry("join-001", "join", None),
    );
    state.before_final = Some(Box::new(|state| {
        state.frame_at = Some(Instant::now() - Duration::from_millis(1001))
    }));
    let received = Instant::now();
    let reply = state.execute(spec, Screen::Menu, input, received, Duration::ZERO);
    assert!(
        received.elapsed() < Duration::from_secs(2),
        "request-deadline fixture did not discriminate UI age"
    );
    assert!(
        reply.is_err(),
        "cached frame age was reset to zero after intent"
    );
    assert!(
        state
            .dir
            .join("operations/connection-join-001.json")
            .is_file()
    );
    assert!(loader.client_commands().is_empty());
    clean(state);
}

#[test]
fn last_request_slot_dispatches_but_next_slot_cannot_cancel_it() {
    let (loader, input, mut state, _) = setup();
    let join = real(
        &mut state,
        input,
        Screen::Menu,
        dry("join-last", "join", None),
    );
    state.next = admission::MAX_REQUESTS;
    let value = json!({"operation":join.operation,"action":"join","profile":"isolated-test","dry":false,"preview":join.preview,"confirm":join.confirm});
    fs::write(
        state
            .dir
            .join(format!("request-{:04}.json", admission::MAX_REQUESTS)),
        value.to_string(),
    )
    .unwrap();
    state.frame(Screen::Menu, input).unwrap();
    state.poll(input).unwrap();
    assert_eq!(loader.client_commands().len(), 1);
    assert_eq!(state.next, admission::MAX_REQUESTS + 1);
    let cancel = real(
        &mut state,
        input,
        Screen::Menu,
        dry("cancel-extra", "cancel", Some("join-last")),
    );
    let value = json!({"operation":cancel.operation,"action":"cancel","target_operation":"join-last","dry":false,"preview":cancel.preview,"confirm":cancel.confirm});
    fs::write(
        state
            .dir
            .join(format!("request-{:04}.json", admission::MAX_REQUESTS + 1)),
        value.to_string(),
    )
    .unwrap();
    state.frame(Screen::Menu, input).unwrap();
    assert!(matches!(
        input
            .command_status(state.jobs[0].id.unwrap())
            .unwrap()
            .state,
        CommandState::Pending | CommandState::Applied
    ));
    assert_eq!(loader.client_commands().len(), 1);
    clean(state);
}

#[test]
fn delayed_readiness_keeps_a_full_command_phase_for_owned_leave() {
    let (loader, input, mut state, address) = setup();
    state.started = Instant::now() - Duration::from_secs(120);
    state.frame(Screen::Menu, input).unwrap();
    submit_join(&mut state, input);
    let id = loader.client_commands()[0].0;
    loader.set_command_status(id, Raw::OBSERVED, 0);
    loader.set_command_phase(id, Raw::PHASE_IN_WORLD, 0);
    loader.set_client_snapshot(Snapshot {
        state: Snapshot::FRESH,
        session: 9,
        age_ms: 0,
        server: Str::new(&address),
        ..Snapshot::EMPTY
    });
    state.started = Instant::now() - Duration::from_secs(200);
    state.frame(Screen::InGame, input).unwrap();
    assert!(!state.halted, "human wait consumed the owned Leave budget");
    let spec = real(
        &mut state,
        input,
        Screen::InGame,
        dry("leave-001", "leave", None),
    );
    assert_eq!(
        state
            .execute(spec, Screen::InGame, input, Instant::now(), Duration::ZERO)
            .unwrap()["state"],
        "submitted"
    );
    assert_eq!(loader.client_commands().len(), 2);
    clean(state);
}

#[test]
fn real_request_cannot_start_command_lease_without_accepted_preview() {
    let (loader, input, mut state, _) = setup();
    let mut spec = dry("join-001", "join", None);
    spec.dry = false;
    let (context, _) = state.context(Screen::Menu, input);
    spec.preview = Some(admission::preview(&state.boot, &spec, &context));
    spec.confirm = Some("server-write".into());
    fs::write(state.dir.join(policy::write_gate::FLAG), []).unwrap();
    assert!(
        state
            .execute(spec, Screen::Menu, input, Instant::now(), Duration::ZERO)
            .is_err()
    );
    assert!(loader.client_commands().is_empty());
    assert_eq!(
        fs::read_dir(state.dir.join("operations")).unwrap().count(),
        0
    );
    clean(state);
}

#[test]
fn join_admission_expires_sixty_seconds_after_first_preview_without_reset() {
    let (loader, input, mut state, _) = setup();
    let spec = real(
        &mut state,
        input,
        Screen::Menu,
        dry("join-001", "join", None),
    );
    state.started = Instant::now() - Duration::from_secs(61);
    state.frame(Screen::Menu, input).unwrap();
    assert!(
        state
            .execute(spec, Screen::Menu, input, Instant::now(), Duration::ZERO)
            .is_err()
    );
    assert!(loader.client_commands().is_empty());
    clean(state);
}

#[test]
fn heartbeat_distinguishes_observer_from_started_command_lease() {
    let (loader, input, mut state, _) = setup();
    let rows = || fs::read_to_string(state.dir.join("observations.jsonl")).unwrap();
    let before: Value = serde_json::from_str(rows().lines().last().unwrap()).unwrap();
    assert_eq!(before["lease_model"], "observer_then_commands_v1");
    assert_eq!(before["lease_phase"], "observing");
    assert!(before["command_lease_ms"].is_null());
    assert!(before.get("session_lease_ms").is_none());
    assert!(loader.client_commands().is_empty());
    state
        .execute(
            dry("join-001", "join", None),
            Screen::Menu,
            input,
            Instant::now(),
            Duration::ZERO,
        )
        .unwrap();
    state.heartbeat = None;
    state.frame(Screen::Menu, input).unwrap();
    let text = fs::read_to_string(state.dir.join("observations.jsonl")).unwrap();
    let after: Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
    assert_eq!(after["lease_phase"], "commands");
    assert!(after["command_lease_ms"].is_u64());
    assert_eq!(
        text.lines()
            .filter(|line| line.contains("command_lease_started"))
            .count(),
        1
    );
    assert!(loader.client_commands().is_empty());
    clean(state);
}

#[test]
fn refused_preview_does_not_start_command_authority() {
    let (loader, input, mut state, _) = setup();
    assert!(
        state
            .execute(
                dry("join-001", "join", None),
                Screen::Other,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err()
    );
    assert_eq!(state.lease.phase(), "observing");
    assert!(
        state
            .lease
            .command_age(state.started.elapsed())
            .unwrap()
            .is_none()
    );
    assert!(loader.client_commands().is_empty());
    clean(state);
}

#[test]
fn unconfirmed_lease_start_record_halts_without_native_submission() {
    let (loader, input, mut state, _) = setup();
    state.journal = File::open(state.dir.join("observations.jsonl")).unwrap();
    assert!(
        state
            .execute(
                dry("join-001", "join", None),
                Screen::Menu,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err()
    );
    assert!(state.halted);
    assert!(
        state
            .execute(
                dry("join-002", "join", None),
                Screen::Menu,
                input,
                Instant::now(),
                Duration::ZERO
            )
            .is_err()
    );
    assert!(loader.client_commands().is_empty());
    clean(state);
}

#[test]
fn command_lease_expiring_after_intent_burns_identity_without_dispatch() {
    let (loader, input, mut state, _) = setup();
    let spec = real(
        &mut state,
        input,
        Screen::Menu,
        dry("join-001", "join", None),
    );
    let original_start = state.started;
    state.before_final = Some(Box::new(|s| {
        s.started = Instant::now() - Duration::from_secs(181)
    }));
    let error = state
        .execute(
            spec.clone(),
            Screen::Menu,
            input,
            Instant::now(),
            Duration::ZERO,
        )
        .unwrap_err();
    assert_eq!(error, "command_lease_expired");
    assert!(
        state
            .dir
            .join("operations/connection-join-001.json")
            .is_file()
    );
    assert!(loader.client_commands().is_empty());
    state.started = original_start;
    assert!(
        state
            .execute(spec, Screen::Menu, input, Instant::now(), Duration::ZERO)
            .is_err()
    );
    assert!(loader.client_commands().is_empty());
    clean(state);
}
