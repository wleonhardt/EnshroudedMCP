use policy::client_connection::{self as connection, Context, Owned, Screen};
use serde_json::{Value, json};
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

fn spec(value: Value) -> connection::Spec {
    connection::check(value.as_object().unwrap()).unwrap()
}
fn menu() -> Context {
    Context {
        screen: Screen::Menu,
        age_ms: Some(0),
        sequence: 1,
        native_session: Some(0),
        native_fresh: false,
        native_age_ms: None,
        peer_matches: None,
        owned: Owned::Idle,
    }
}
fn join(real: bool) -> connection::Spec {
    let mut value = spec(json!({"operation":"join-001","action":"join","profile":"isolated-test"}));
    value.dry = !real;
    value
}

#[test]
fn requests_default_dry_and_exclude_credentials_and_targets() {
    assert!(spec(json!({"operation":"join-001","action":"join","profile":"isolated-test"})).dry);
    for field in ["password", "server", "address", "credential", "character"] {
        let mut value = json!({"operation":"join-001","action":"join","profile":"isolated-test"});
        value[field] = json!("synthetic-do-not-disclose");
        let error = connection::check(value.as_object().unwrap()).unwrap_err();
        assert!(error.contains("unknown argument"));
        assert!(!error.contains("synthetic-do-not-disclose"));
    }
    for value in [
        json!({"operation":"JOIN","action":"join","profile":"isolated-test"}),
        json!({"operation":"../join","action":"join","profile":"isolated-test"}),
        json!({"operation":"j","action":"join"}),
        json!({"operation":"j","action":"leave","profile":"isolated-test"}),
        json!({"operation":"j","action":"cancel"}),
        json!({"operation":"j","action":"join","profile":"p","dry":"false"}),
    ] {
        assert!(connection::check(value.as_object().unwrap()).is_err());
    }
}

#[test]
fn private_profile_is_loopback_only_and_debug_and_peer_results_are_bounded() {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, 7)).to_string();
    let profile = connection::profile(&json!({"profile":"isolated-test","query":address})).unwrap();
    assert_eq!(profile.query(), address);
    assert!(!format!("{profile:?}").contains(&address));
    assert_eq!(profile.matches_peer(None), None);
    assert_eq!(profile.matches_peer(Some("not-an-address")), None);
    assert_eq!(profile.matches_peer(Some(&address)), Some(true));
    assert_eq!(
        profile.matches_peer(Some(
            &SocketAddr::from((Ipv4Addr::LOCALHOST, 8)).to_string()
        )),
        Some(false)
    );
    for query in [
        SocketAddr::from((Ipv4Addr::UNSPECIFIED, 7)).to_string(),
        SocketAddr::from((Ipv4Addr::LOCALHOST, 0)).to_string(),
        "not-an-address".into(),
    ] {
        assert!(connection::profile(&json!({"profile":"isolated-test","query":query})).is_err());
    }
    assert!(
        connection::profile(
            &json!({"profile":"isolated-test","query":address,"password":"synthetic"})
        )
        .is_err()
    );
}

#[test]
fn join_requires_fresh_menu_known_idle_and_its_allowlisted_profile() {
    let s = join(false);
    let base = menu();
    assert!(connection::applicable(&s, &base, "isolated-test").is_ok());
    for changed in [
        Context {
            screen: Screen::InGame,
            ..base.clone()
        },
        Context {
            age_ms: None,
            ..base.clone()
        },
        Context {
            age_ms: Some(1001),
            ..base.clone()
        },
        Context {
            native_session: Some(2),
            ..base.clone()
        },
        Context {
            owned: Owned::Pending("other".into()),
            ..base.clone()
        },
        Context {
            owned: Owned::Unknown,
            ..base.clone()
        },
    ] {
        assert!(connection::applicable(&s, &changed, "isolated-test").is_err());
    }
    assert!(connection::applicable(&s, &base, "other").is_err());
    // Menu admission is distinct from the later SDK connection executor guard.
    assert!(
        connection::applicable(
            &s,
            &Context {
                native_session: None,
                ..base
            },
            "isolated-test"
        )
        .is_ok()
    );
}

#[test]
fn leave_requires_own_observed_join_and_fresh_matching_native_context() {
    let s = spec(json!({"operation":"leave-001","action":"leave"}));
    let good = Context {
        screen: Screen::InGame,
        age_ms: Some(0),
        sequence: 2,
        native_session: Some(3),
        native_fresh: true,
        native_age_ms: Some(0),
        peer_matches: Some(true),
        owned: Owned::Joined {
            operation: "join-001".into(),
            session: 3,
        },
    };
    assert!(connection::applicable(&s, &good, "isolated-test").is_ok());
    for c in [
        menu(),
        Context {
            owned: Owned::Idle,
            ..good.clone()
        },
        Context {
            native_fresh: false,
            ..good.clone()
        },
        Context {
            native_age_ms: None,
            ..good.clone()
        },
        Context {
            native_age_ms: Some(1001),
            ..good.clone()
        },
        Context {
            peer_matches: None,
            ..good.clone()
        },
        Context {
            peer_matches: Some(false),
            ..good.clone()
        },
        Context {
            native_session: Some(0),
            ..good.clone()
        },
        Context {
            native_session: Some(4),
            ..good.clone()
        },
        Context {
            screen: Screen::Other,
            ..good
        },
    ] {
        assert!(connection::applicable(&s, &c, "isolated-test").is_err());
    }
}

fn directory() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap();
    let parent = root.join("gamedata/qualification-tests/connection-policy");
    std::fs::create_dir_all(&parent).unwrap();
    assert!(parent.canonicalize().unwrap().starts_with(&root));
    let dir = parent.join(format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    dir
}

#[test]
fn durable_identity_refuses_dry_duplicates_conflicts_and_path_escape() {
    let dir = directory();
    let ctx = menu();
    assert!(connection::reserve(&dir, "boot-1", &join(false), &ctx).is_err());
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    let mut s = join(true);
    connection::reserve(&dir, "boot-1", &s, &ctx).unwrap();
    let path = dir.join("connection-join-001.json");
    let before = std::fs::read(&path).unwrap();
    assert!(connection::reserve(&dir, "boot-1", &s, &ctx).is_err());
    s.action = connection::Action::Leave;
    assert!(connection::reserve(&dir, "boot-2", &s, &ctx).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    s.operation = "../outside".into();
    assert!(connection::reserve(&dir, "boot-1", &s, &ctx).is_err());
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn reservation_capacity_and_unexpected_members_refuse_without_new_intent() {
    let dir = directory();
    let ctx = menu();
    std::fs::write(dir.join("foreign"), b"keep").unwrap();
    assert!(connection::reserve(&dir, "boot-1", &join(true), &ctx).is_err());
    assert_eq!(std::fs::read(dir.join("foreign")).unwrap(), b"keep");
    std::fs::remove_file(dir.join("foreign")).unwrap();
    for i in 0..connection::MAX_OPERATIONS {
        let mut s = join(true);
        s.operation = format!("operation-{i}");
        connection::reserve(&dir, "boot-1", &s, &ctx).unwrap();
    }
    assert!(connection::reserve(&dir, "boot-1", &join(true), &ctx).is_err());
    assert_eq!(
        std::fs::read_dir(&dir).unwrap().count(),
        connection::MAX_OPERATIONS
    );
    for i in 0..connection::MAX_OPERATIONS {
        std::fs::remove_file(dir.join(format!("connection-operation-{i}.json"))).unwrap();
    }
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn restart_cannot_replace_exclusive_session_marker() {
    let dir = directory();
    connection::start_session(&dir, "boot-1").unwrap();
    let before = std::fs::read(dir.join("session.json")).unwrap();
    assert!(connection::start_session(&dir, "boot-2").is_err());
    assert_eq!(std::fs::read(dir.join("session.json")).unwrap(), before);
    std::fs::remove_file(dir.join("session.json")).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn exact_header_and_bounded_lease_do_not_grant_provider_fidelity_or_cleanup() {
    assert!(connection::provider(12, 16, 4096, 4096).is_ok());
    for (major, minor, bytes, compiled) in [
        (12, 1, 4096, 4096),
        (13, 16, 4096, 4096),
        (12, 16, 4095, 4096),
        (12, 16, 0, 0),
    ] {
        assert!(connection::provider(major, minor, bytes, compiled).is_err());
    }
    assert!(connection::lease(Duration::from_secs(179)).is_ok());
    assert!(connection::lease(Duration::from_secs(180)).is_err());
}

#[test]
fn retained_memory_identity_and_capacity_do_not_depend_on_disk_files() {
    assert!(connection::new_operation("new", std::iter::empty()).is_ok());
    assert!(connection::new_operation("same", ["same"]).is_err());
    let known: Vec<_> = (0..connection::MAX_OPERATIONS)
        .map(|i| format!("old-{i}"))
        .collect();
    assert!(connection::new_operation("new", known[..7].iter().map(String::as_str)).is_ok());
    assert!(connection::new_operation("new", known.iter().map(String::as_str)).is_err());
}

#[test]
fn ownership_requires_ordered_noncontradictory_retained_observations() {
    use connection::{Action, OperationObservation as O};
    let join = Action::Join("p".into());
    let leave = Action::Leave;
    let cancel = Action::Cancel("join".into());
    assert_eq!(
        connection::ownership(Screen::Menu, std::iter::empty()),
        Owned::Idle
    );
    assert_eq!(
        connection::ownership(
            Screen::Menu,
            [O {
                operation: "join",
                action: &join,
                pending: true,
                joined_session: None,
                left: false
            }]
        ),
        Owned::Pending("join".into())
    );
    let joined = || O {
        operation: "join",
        action: &join,
        pending: false,
        joined_session: Some(9),
        left: false,
    };
    assert_eq!(
        connection::ownership(Screen::InGame, [joined()]),
        Owned::Joined {
            operation: "join".into(),
            session: 9
        }
    );
    assert_eq!(
        connection::ownership(
            Screen::Menu,
            [
                joined(),
                O {
                    operation: "leave",
                    action: &leave,
                    pending: false,
                    joined_session: None,
                    left: true
                }
            ]
        ),
        Owned::Idle
    );
    assert_eq!(
        connection::ownership(
            Screen::Menu,
            [O {
                operation: "leave",
                action: &leave,
                pending: false,
                joined_session: None,
                left: true
            }]
        ),
        Owned::Unknown
    );
    assert_eq!(
        connection::ownership(
            Screen::Menu,
            [
                joined(),
                O {
                    operation: "again",
                    action: &join,
                    pending: true,
                    joined_session: None,
                    left: false
                }
            ]
        ),
        Owned::Unknown
    );
    assert_eq!(
        connection::ownership(
            Screen::Menu,
            [O {
                operation: "bad",
                action: &join,
                pending: true,
                joined_session: Some(9),
                left: false
            }]
        ),
        Owned::Unknown
    );
    assert_eq!(
        connection::ownership(
            Screen::Menu,
            [
                joined(),
                O {
                    operation: "cancel",
                    action: &cancel,
                    pending: false,
                    joined_session: None,
                    left: false
                }
            ]
        ),
        Owned::Unknown
    );
}

#[test]
fn first_session_binding_requires_fresh_matching_observations_and_never_rebinds() {
    let good = Context {
        screen: Screen::InGame,
        native_session: Some(9),
        native_fresh: true,
        native_age_ms: Some(0),
        peer_matches: Some(true),
        ..menu()
    };
    assert_eq!(connection::bind_join_session(None, true, &good), Some(9));
    assert_eq!(connection::bind_join_session(None, false, &good), None);
    for changed in [
        Context {
            sequence: 0,
            ..good.clone()
        },
        Context {
            age_ms: Some(1001),
            ..good.clone()
        },
        Context {
            native_age_ms: None,
            ..good.clone()
        },
        Context {
            peer_matches: None,
            ..good.clone()
        },
        Context {
            native_session: Some(0),
            ..good.clone()
        },
    ] {
        assert_eq!(connection::bind_join_session(None, true, &changed), None);
    }
    assert_eq!(
        connection::bind_join_session(
            Some(9),
            true,
            &Context {
                native_session: Some(10),
                ..good
            }
        ),
        Some(9)
    );
}

#[test]
fn bounded_regular_json_preserves_age_and_refuses_bad_or_future_files() {
    let dir = directory();
    let path = dir.join("request.json");
    assert!(connection::read_object(&path).unwrap().is_none());
    assert!(connection::read_object(&dir).is_err());
    std::fs::write(&path, b"{}").unwrap();
    let file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
    file.set_modified(std::time::SystemTime::now() - Duration::from_secs(3))
        .unwrap();
    assert!(connection::read_object(&path).unwrap().unwrap().1 >= Duration::from_secs(3));
    file.set_modified(std::time::SystemTime::now() + Duration::from_secs(30))
        .unwrap();
    assert!(connection::read_object(&path).is_err());
    drop(file);
    std::fs::write(&path, vec![b'x'; 4097]).unwrap();
    assert!(connection::read_object(&path).is_err());
    std::fs::write(&path, b"invalid").unwrap();
    assert!(connection::read_object(&path).is_err());
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn cancellation_targets_only_retained_pending_join_and_never_implies_leave() {
    let s = spec(json!({"operation":"cancel-001","action":"cancel","target_operation":"join-001"}));
    let good = Context {
        screen: Screen::Other,
        owned: Owned::Pending("join-001".into()),
        ..menu()
    };
    assert!(connection::applicable(&s, &good, "isolated-test").is_ok());
    for owned in [
        Owned::Idle,
        Owned::Unknown,
        Owned::Joined {
            operation: "join-001".into(),
            session: 3,
        },
        Owned::Pending("other".into()),
    ] {
        assert!(
            connection::applicable(
                &s,
                &Context {
                    owned,
                    ..good.clone()
                },
                "isolated-test"
            )
            .is_err()
        );
    }
}

#[test]
fn real_gate_rechecks_preview_flag_context_and_dispatch_deadline() {
    let ctx = menu();
    let mut s = join(true);
    let preview = connection::preview("boot-1", &s, &ctx);
    s.preview = Some(preview);
    s.confirm = Some("server-write".into());
    assert!(
        connection::authorize(
            "boot-1",
            &s,
            &ctx,
            "isolated-test",
            Some(Duration::ZERO),
            Duration::ZERO
        )
        .is_ok()
    );
    assert!(
        connection::authorize("boot-1", &s, &ctx, "isolated-test", None, Duration::ZERO).is_err()
    );
    assert!(
        connection::authorize(
            "boot-1",
            &s,
            &ctx,
            "isolated-test",
            Some(Duration::from_secs(1800)),
            Duration::ZERO
        )
        .is_err()
    );
    assert!(
        connection::authorize(
            "boot-1",
            &s,
            &ctx,
            "isolated-test",
            Some(Duration::ZERO),
            Duration::from_secs(2)
        )
        .is_err()
    );
    assert!(
        connection::authorize(
            "boot-2",
            &s,
            &ctx,
            "isolated-test",
            Some(Duration::ZERO),
            Duration::ZERO
        )
        .is_err()
    );
    assert!(
        connection::authorize(
            "boot-1",
            &s,
            &Context {
                sequence: 2,
                ..ctx.clone()
            },
            "isolated-test",
            Some(Duration::ZERO),
            Duration::ZERO
        )
        .is_err()
    );
    s.confirm = Some("wrong".into());
    assert!(
        connection::authorize(
            "boot-1",
            &s,
            &ctx,
            "isolated-test",
            Some(Duration::ZERO),
            Duration::ZERO
        )
        .is_err()
    );
    assert!(
        connection::authorize(
            "boot-1",
            &join(false),
            &ctx,
            "isolated-test",
            None,
            Duration::ZERO
        )
        .is_ok()
    );
}
