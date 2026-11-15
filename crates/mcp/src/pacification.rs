//! One typed gameplay switch, called only on the server game thread.
use enshwick::game_settings::GameSettings;
use policy::{pacification::Spec, write_gate};
use serde_json::{Value, json};
use std::{fs::OpenOptions, io::Write, path::Path};

use policy::pacification::RECORD_MAX_BYTES;

pub(crate) fn gate(spec: &Spec, dir: Option<&Path>) -> Result<(), String> {
    write_gate::check(
        spec.dry || spec.enabled.is_none(),
        true,
        dir.and_then(|d| write_gate::flag_age(&d.join(write_gate::FLAG))),
        spec.confirm.as_deref(),
    )
}

fn journal_size(dir: &Path) -> Result<u64, String> {
    let path = dir.join("journal.jsonl");
    Ok(match std::fs::metadata(&path) {
        Ok(meta) if meta.is_file() => meta.len(),
        Ok(_) => return Err("pacification journal must be a regular file".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
        Err(e) => return Err(e.to_string()),
    })
}

fn append(dir: &Path, record: Value) -> Result<(), String> {
    let bytes = record.to_string() + "\n";
    policy::pacification::check_record_size(bytes.len())?;
    let size = journal_size(dir)?;
    policy::construction::check_journal_size(size, bytes.len())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("journal.jsonl"))
        .map_err(|e| e.to_string())?;
    file.write_all(bytes.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("pacification journal durability failed: {e}"))
}

pub(crate) fn run(
    settings: GameSettings,
    spec: Spec,
    session: &str,
    dir: Option<&Path>,
) -> Result<Value, String> {
    run_with(
        spec,
        session,
        dir,
        || {
            settings
                .read()
                .map(|(settings, _)| settings.pacify_all_enemies)
        },
        |enabled| {
            settings
                .change(|settings| settings.pacify_all_enemies = enabled)
                .map(|_| ())
                .map_err(|error| error.to_string())
        },
    )
}

fn run_with(
    spec: Spec,
    session: &str,
    dir: Option<&Path>,
    mut read: impl FnMut() -> Option<bool>,
    mut change: impl FnMut(bool) -> Result<(), String>,
) -> Result<Value, String> {
    gate(&spec, dir)?;
    let before = read().ok_or("game settings unavailable")?;
    if let Some(expected) = spec.expected {
        policy::pacification::check_current(expected, before)?;
    }
    if let Some(preview_session) = &spec.session {
        policy::construction::check_session(preview_session, session)?;
    }
    let mut result = json!({"session":session,"enabled":before,"requested":spec.enabled,
        "state":"observed","world_mutated":false,"combat_safety_verified":false,
        "scope":"server gameplay pacify_all_enemies only; not invulnerability",
        "persistence":"change only, not keep; configuration saves may persist it; restore explicitly"});
    let Some(enabled) = spec.enabled else {
        return Ok(result);
    };
    if spec.dry {
        result["state"] = json!("dry");
        return Ok(result);
    }
    if enabled == before {
        result["state"] = json!("already_equal");
        return Ok(result);
    }
    let dir = dir.ok_or("mod folder unavailable")?;
    // Reserve both bounded records before mutation, not only the smaller intent.
    policy::construction::check_journal_size(journal_size(dir)?, 2 * RECORD_MAX_BYTES)?;
    append(
        dir,
        json!({"kind":"pacification_intent","session":session,"before":before,"requested":enabled}),
    )?;
    gate(&spec, Some(dir))?;
    if let Err(error) = change(enabled) {
        append(
            dir,
            json!({"kind":"pacification_outcome","session":session,"state":"command_error_unknown"}),
        )?;
        return Err(format!(
            "pacification command failed; inspect current state before another request: {error}"
        ));
    }
    let observed = read();
    result["state"] = json!(match observed {
        Some(value) if value == enabled => "changed_readback",
        Some(_) => "readback_mismatch",
        None => "readback_unavailable",
    });
    result["world_mutated"] = json!(true);
    result["enabled"] = json!(observed);
    result["readback_equal"] = json!(observed == Some(enabled));
    append(dir, json!({"kind":"pacification_outcome","session":session,"result":result}))
        .map_err(|e| format!("pacification may have changed; outcome journal failed; read current state, do not blindly retry: {e}"))?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::Cell,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap();
            let path = root.join("target").join(format!(
                "pacification-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            fs::write(path.join(write_gate::FLAG), "").unwrap();
            Self(path)
        }
        fn records(&self) -> Vec<Value> {
            fs::read_to_string(self.0.join("journal.jsonl"))
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn spec(value: Value) -> Spec {
        policy::pacification::check(value.as_object().unwrap()).unwrap()
    }
    fn real() -> Spec {
        spec(
            json!({"enabled":true,"expected":false,"session":"active","dry":false,"confirm":"server-write"}),
        )
    }

    #[test]
    fn read_and_dry_never_change_or_journal() {
        for (request, state) in [(json!({}), "observed"), (json!({"enabled":true}), "dry")] {
            let result = run_with(
                spec(request),
                "active",
                None,
                || Some(false),
                |_| panic!("dry changed settings"),
            )
            .unwrap();
            assert_eq!(result["state"], state);
            assert_eq!(result["world_mutated"], false);
        }
    }

    #[test]
    fn nonregular_intent_journal_refuses_before_change() {
        let fixture = Fixture::new();
        fs::create_dir(fixture.0.join("journal.jsonl")).unwrap();
        assert!(
            run_with(
                real(),
                "active",
                Some(&fixture.0),
                || Some(false),
                |_| panic!("changed before durable intent")
            )
            .is_err()
        );
    }

    #[cfg(windows)]
    #[test]
    fn intent_append_sharing_failure_prevents_native_change() {
        use std::os::windows::fs::OpenOptionsExt;
        let fixture = Fixture::new();
        let path = fixture.0.join("journal.jsonl");
        fs::write(&path, "").unwrap();
        let hold = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&path)
            .unwrap();
        assert!(
            run_with(
                real(),
                "active",
                Some(&fixture.0),
                || Some(false),
                |_| panic!("changed after failed intent append")
            )
            .is_err()
        );
        assert_eq!(fs::metadata(path).unwrap().len(), 0);
        drop(hold);
    }

    #[test]
    fn reserves_outcome_headroom_before_intent_or_change() {
        let fixture = Fixture::new();
        let path = fixture.0.join("journal.jsonl");
        let file = fs::File::create(&path).unwrap();
        let size = (policy::construction::JOURNAL_MAX_BYTES - RECORD_MAX_BYTES) as u64;
        file.set_len(size).unwrap();
        drop(file);
        assert!(
            run_with(
                real(),
                "active",
                Some(&fixture.0),
                || Some(false),
                |_| panic!("changed without outcome headroom")
            )
            .is_err()
        );
        assert_eq!(fs::metadata(path).unwrap().len(), size);
    }

    #[test]
    fn stale_expected_or_session_refuses_without_intent_or_change() {
        for (observed, session) in [(true, "active"), (false, "other")] {
            let fixture = Fixture::new();
            assert!(
                run_with(
                    real(),
                    session,
                    Some(&fixture.0),
                    || Some(observed),
                    |_| panic!("stale preview changed settings")
                )
                .is_err()
            );
            assert!(!fixture.0.join("journal.jsonl").exists());
        }
    }

    #[test]
    fn removed_flag_after_first_gate_refuses_after_intent_before_change() {
        let fixture = Fixture::new();
        let result = run_with(
            real(),
            "active",
            Some(&fixture.0),
            || {
                fs::remove_file(fixture.0.join(write_gate::FLAG)).unwrap();
                Some(false)
            },
            |_| panic!("changed after write flag was removed"),
        );
        assert!(result.is_err());
        let records = fixture.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["kind"], "pacification_intent");
    }

    #[test]
    fn successful_change_has_exact_boolean_readback_and_durable_outcome() {
        let fixture = Fixture::new();
        let current = Cell::new(false);
        let calls = Cell::new(0);
        let result = run_with(
            real(),
            "active",
            Some(&fixture.0),
            || Some(current.get()),
            |enabled| {
                calls.set(calls.get() + 1);
                current.set(enabled);
                Ok(())
            },
        )
        .unwrap();
        assert!(current.get());
        assert_eq!(calls.get(), 1);
        assert_eq!(result["state"], "changed_readback");
        assert_eq!(result["readback_equal"], true);
        assert_eq!(result["combat_safety_verified"], false);
        let records = fixture.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["before"], false);
        assert_eq!(records[0]["requested"], true);
        assert_eq!(records[1]["result"], result);
    }

    #[test]
    fn unavailable_or_different_readback_never_claims_verified_change() {
        for after in [None, Some(false)] {
            let fixture = Fixture::new();
            let first = Cell::new(true);
            let result = run_with(
                real(),
                "active",
                Some(&fixture.0),
                || {
                    if first.replace(false) {
                        Some(false)
                    } else {
                        after
                    }
                },
                |_| Ok(()),
            )
            .unwrap();
            assert_eq!(result["readback_equal"], false);
            assert_eq!(
                result["state"],
                if after.is_none() {
                    "readback_unavailable"
                } else {
                    "readback_mismatch"
                }
            );
            assert_eq!(fixture.records().len(), 2);
        }
    }

    #[test]
    fn native_error_is_retained_as_unknown_without_retry() {
        let fixture = Fixture::new();
        let calls = Cell::new(0);
        assert!(
            run_with(
                real(),
                "active",
                Some(&fixture.0),
                || Some(false),
                |_| {
                    calls.set(calls.get() + 1);
                    Err("native error".into())
                }
            )
            .is_err()
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(fixture.records()[1]["state"], "command_error_unknown");
    }

    #[test]
    fn already_equal_still_requires_gate_but_never_changes_or_journals() {
        let fixture = Fixture::new();
        let mut request = real();
        request.expected = Some(true);
        let result = run_with(
            request.clone(),
            "active",
            Some(&fixture.0),
            || Some(true),
            |_| panic!("already equal dispatched"),
        )
        .unwrap();
        assert_eq!(result["state"], "already_equal");
        assert!(!fixture.0.join("journal.jsonl").exists());
        fs::remove_file(fixture.0.join(write_gate::FLAG)).unwrap();
        assert!(
            run_with(
                request,
                "active",
                Some(&fixture.0),
                || panic!("read before missing gate rejection"),
                |_| panic!("changed without gate")
            )
            .is_err()
        );
    }
}
