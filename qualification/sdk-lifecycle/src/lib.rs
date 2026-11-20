//! Inactive lifecycle consumer contracts. No product endpoint or credentials.
#![cfg(test)]
use enshwick::{
    Load, Mod, Off, WorldError,
    abi::{ClientCommandStatus as Raw, ClientSnapshot as Snapshot, Side, Status, Str},
    input::{ClientCommand, CommandAction, CommandState, CommandStatus, Input, Password},
};
use serde_json::{Value, json};
use std::sync::Mutex;

static INPUT: Mutex<Option<Input>> = Mutex::new(None);
struct Driver;
impl Mod for Driver {
    fn init(load: &mut Load<'_>) -> Result<Self, Off> {
        *INPUT.lock().unwrap() = Some(load.input()?);
        Ok(Self)
    }
}
enshwick::test::declare_mod!(Driver, "id = \"test.lifecycle\"\nside = \"both\"");

fn start() -> (enshwick::test::Installed, Input) {
    let loader = enshwick::test::FakeHost::new()
        .side(Side::CLIENT)
        .player_input()
        .install();
    // SAFETY: these are this module's generated exports, installed on its fake.
    let status = unsafe { loader.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) };
    assert_eq!(status, Status::OK);
    (loader, INPUT.lock().unwrap().unwrap())
}

fn join() -> ClientCommand {
    ClientCommand::new(
        CommandAction::Join {
            server: "127.0.0.1:15637".into(),
            password: Some(Password::new("synthetic-fixture-password")),
        },
        0,
    )
}

// Deliberately expose neither the address nor the credential. This is a consumer
// prototype, not a durable operation store, admission gate or MCP implementation.
fn observation(input: Input, status: CommandStatus) -> Value {
    let (state, reason) = match status.state {
        CommandState::Pending => ("pending", None),
        CommandState::Applied => ("applied", None),
        CommandState::Observed => ("observed", None),
        CommandState::Cancelled(reason) => ("cancelled", Some(format!("{reason:?}"))),
        CommandState::Unavailable => ("unavailable", None),
        CommandState::Failed(reason) => ("failed", Some(format!("{reason:?}"))),
        _ => ("unknown", None),
    };
    let (peer, snapshot_error, snapshot_metadata) = match input.client_snapshot() {
        Ok(snapshot) => (
            snapshot.server().map(|s| s == "127.0.0.1:15637"),
            None,
            Some(
                json!({"state":format!("{:?}",snapshot.state()).to_ascii_lowercase(),
                "session":snapshot.session(),"age_ms":snapshot.age_ms()}),
            ),
        ),
        Err(error) => (None, Some(format!("{error:?}")), None),
    };
    json!({"native_state":state,"native_reason":reason,
        "native_phase":status.phase.map(|p|format!("{p:?}")),"native_age_ms":status.age_ms,
        "snapshot_server_matches_fixture":peer,"snapshot_read_error":snapshot_error,"snapshot_metadata":snapshot_metadata,
        "independent_roster_verified":false,"character_verified":false,
        "rendered_verified":false,"workflow_accepted":false,"automatic_retry":false})
}

#[test]
fn abi_and_debug_redaction_are_pinned_without_real_credentials() {
    assert_eq!(
        (enshwick::abi::ABI_MAJOR, enshwick::abi::ABI_MINOR),
        (12, 11)
    );
    let password = Password::new("synthetic-fixture-password");
    assert_eq!(format!("{password:?}"), "Password(..)");
    assert!(!format!("{:?}", join()).contains("synthetic-fixture-password"));
}

#[test]
fn phase_progress_and_observed_never_become_peer_or_workflow_acceptance() {
    let (loader, input) = start();
    let id = input.client_command(&join()).unwrap();
    for (phase, expected) in [
        (Raw::PHASE_QUEUED, "Queued"),
        (Raw::PHASE_MENU, "Menu"),
        (Raw::PHASE_CONNECTING, "Connecting"),
        (Raw::PHASE_LOADING, "Loading"),
        (Raw::PHASE_IN_WORLD, "InWorld"),
    ] {
        loader.set_command_status(1, Raw::APPLIED, 0);
        loader.set_command_phase(1, phase, 0);
        let observed = observation(input, input.command_status(id).unwrap());
        assert_eq!(observed["native_phase"], expected);
        assert_eq!(observed["workflow_accepted"], false);
        assert_eq!(observed["snapshot_server_matches_fixture"], Value::Null);
    }
    loader.set_command_status(1, Raw::OBSERVED, 0);
    let observed = observation(input, input.command_status(id).unwrap());
    assert_eq!(observed["native_state"], "observed");
    assert_eq!(observed["workflow_accepted"], false);
    assert_eq!(loader.client_commands().len(), 1);
}

#[test]
fn missing_matching_and_different_snapshot_peers_remain_distinct() {
    let (loader, input) = start();
    assert!(input.client_snapshot().unwrap().server().is_none());
    let _ = loader.draw_frame();
    let id = input.client_command(&join()).unwrap();
    loader.set_command_status(1, Raw::OBSERVED, 0);
    loader.set_command_phase(1, Raw::PHASE_IN_WORLD, 0);
    for (peer, expected) in [
        ("", Value::Null),
        ("127.0.0.1:15637", json!(true)),
        ("127.0.0.1:15638", json!(false)),
    ] {
        loader.set_client_snapshot(Snapshot {
            session: 9,
            server: Str::new(peer),
            ..Snapshot::EMPTY
        });
        let observed = observation(input, input.command_status(id).unwrap());
        assert_eq!(observed["snapshot_server_matches_fixture"], expected);
        assert_eq!(observed["snapshot_read_error"], Value::Null);
        assert_eq!(observed["workflow_accepted"], false);
        assert!(!observed.to_string().contains("127.0.0.1"));
        assert!(!observed.to_string().contains("synthetic-fixture-password"));
    }
}

#[test]
fn all_failure_and_unknown_phase_values_survive_without_retry() {
    let (loader, input) = start();
    for (failure, expected) in [
        (Raw::WRONG_PASSWORD, "WrongPassword"),
        (Raw::FULL, "Full"),
        (Raw::NOT_FOUND, "NotFound"),
        (Raw::RESERVED, "Reserved"),
        (Raw::VERSION_MISMATCH, "VersionMismatch"),
        (Raw::REFUSED, "Refused"),
        (Raw::JOIN_TIMED_OUT, "TimedOut"),
        (Raw::UNREACHABLE, "Unreachable"),
        (Raw::UNKNOWN, "Unknown"),
        (u32::MAX, "Other(4294967295)"),
    ] {
        let id = input.client_command(&join()).unwrap();
        let raw_id = loader.client_commands().last().unwrap().0;
        loader.set_command_status(raw_id, Raw::FAILED, 0);
        loader.set_command_phase(raw_id, u32::MAX, failure);
        let before = loader.client_commands().len();
        for _ in 0..2 {
            let observed = observation(input, input.command_status(id).unwrap());
            assert_eq!(observed["native_state"], "failed");
            assert_eq!(observed["native_reason"], expected);
            assert_eq!(observed["native_phase"], "Other(4294967295)");
            assert_eq!(observed["automatic_retry"], false);
        }
        assert_eq!(loader.client_commands().len(), before);
    }
}

#[test]
fn cancellation_never_proves_world_exit_or_submits_a_replacement() {
    let (loader, input) = start();
    let id = input.client_command(&join()).unwrap();
    loader.set_command_status(1, Raw::APPLIED, 0);
    loader.set_command_phase(1, Raw::PHASE_LOADING, 0);
    input.cancel_command(id).unwrap();
    let observed = observation(input, input.command_status(id).unwrap());
    assert_eq!(observed["native_state"], "cancelled");
    assert_eq!(observed["native_reason"], "ByMod");
    assert_eq!(observed["workflow_accepted"], false);
    assert_eq!(loader.client_commands().len(), 1);
}

#[test]
fn join_and_leave_admission_do_not_implicitly_switch_worlds() {
    let (loader, input) = start();
    assert!(matches!(
        input.client_command(&ClientCommand::new(CommandAction::Leave, 0)),
        Err(WorldError::Refused)
    ));
    let id = input.client_command(&join()).unwrap();
    assert!(matches!(
        input.client_command(&join()),
        Err(WorldError::Refused)
    ));
    assert!(matches!(
        input.client_command(&ClientCommand::new(CommandAction::Leave, 0)),
        Err(WorldError::Refused)
    ));
    input.cancel_command(id).unwrap();
    loader.set_client_snapshot(Snapshot {
        session: 2,
        ..Snapshot::EMPTY
    });
    assert!(matches!(
        input.client_command(&join()),
        Err(WorldError::Refused)
    ));
    let leave = input
        .client_command(&ClientCommand::new(CommandAction::Leave, 2))
        .unwrap();
    let raw_id = loader.client_commands().last().unwrap().0;
    loader.set_command_status(raw_id, Raw::OBSERVED, 0);
    loader.set_command_phase(raw_id, Raw::PHASE_LEFT, 0);
    let observed = observation(input, input.command_status(leave).unwrap());
    assert_eq!(observed["native_phase"], "Left");
    assert_eq!(observed["workflow_accepted"], false);
}

#[test]
fn unavailable_snapshot_keeps_command_facts_without_panicking_or_retrying() {
    let (client, input) = start();
    let id = input.client_command(&join()).unwrap();
    client.set_command_status(1, Raw::APPLIED, 0);
    client.set_command_phase(1, Raw::PHASE_MENU, 0);
    let mut status = input.command_status(id).unwrap();
    status.age_ms = 123;
    assert_eq!(client.client_commands().len(), 1);
    drop(client);

    // The published fake's server getter supplies a real typed Unavailable.
    let server = enshwick::test::FakeHost::new()
        .side(Side::SERVER)
        .player_input()
        .install();
    // SAFETY: these are this module's generated exports on its own SDK fake.
    assert_eq!(
        unsafe { server.start(enshwick_mod_query, enshwick_mod_init, enshwick_mod_ready) },
        Status::OK
    );
    let input = INPUT.lock().unwrap().unwrap();
    assert!(matches!(
        input.client_snapshot(),
        Err(WorldError::Unavailable)
    ));
    let observed = observation(input, status);
    assert_eq!(observed["native_state"], "applied");
    assert_eq!(observed["native_phase"], "Menu");
    assert_eq!(observed["native_age_ms"], 123);
    assert_eq!(observed["snapshot_read_error"], "Unavailable");
    assert_eq!(observed["snapshot_metadata"], Value::Null);
    assert_eq!(observed["snapshot_server_matches_fixture"], Value::Null);
    assert_eq!(observed["workflow_accepted"], false);
    assert_eq!(observed["automatic_retry"], false);
    assert_eq!(server.client_commands().len(), 0);
    assert!(!observed.to_string().contains("127.0.0.1"));
    assert!(!observed.to_string().contains("synthetic-fixture-password"));
}

#[test]
fn matching_target_keeps_snapshot_age_state_and_session_without_acceptance() {
    let (loader, input) = start();
    input.client_snapshot().unwrap();
    let _ = loader.draw_frame();
    let id = input.client_command(&join()).unwrap();
    for (state, name, age, session, expected_age) in [
        (Snapshot::FRESH, "fresh", 16, 7, Some(16u64)),
        (Snapshot::STALE, "stale", 4200, 7, Some(4200)),
        (Snapshot::OFF, "off", 4200, 8, Some(4200)),
        (Snapshot::NEVER, "never", u64::MAX, 0, None),
    ] {
        loader.set_client_snapshot(Snapshot {
            state,
            age_ms: age,
            session,
            server: Str::new("127.0.0.1:15637"),
            ..Snapshot::EMPTY
        });
        let value = observation(input, input.command_status(id).unwrap());
        assert_eq!(
            value["snapshot_metadata"],
            json!({"state":name,"age_ms":expected_age,"session":session})
        );
        assert_eq!(value["snapshot_server_matches_fixture"], true);
        assert_eq!(value["snapshot_read_error"], Value::Null);
        assert_eq!(value["workflow_accepted"], false);
        assert_eq!(value["independent_roster_verified"], false);
        assert_eq!(value["character_verified"], false);
        assert_eq!(value["automatic_retry"], false);
        assert_eq!(loader.client_commands().len(), 1);
        assert!(!value.to_string().contains("127.0.0.1"));
        assert!(!value.to_string().contains("synthetic-fixture-password"));
    }
}
