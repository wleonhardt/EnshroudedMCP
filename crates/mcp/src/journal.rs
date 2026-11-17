//! Serialized shared journal appends and session-local recovery allowances.
use policy::construction::RecoveryReservations;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
// Directories come from the loader, never tool arguments.
static STATES: OnceLock<Mutex<BTreeMap<PathBuf, RecoveryReservations>>> = OnceLock::new();
pub(crate) fn tool() -> policy::mcp::Tool {
    policy::mcp::Tool {
        name: "construction_budget",
        description: "Read journal bytes and current session-local retained recovery allowances. No owner IDs, world reads, capacity reservation, native undo-storage observation or write authorization.",
        schema: serde_json::json!({"type":"object","additionalProperties":false,"properties":{}}),
    }
}
pub(crate) fn report(
    dir: &Path,
    args: &serde_json::Map<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    policy::args::check(&tool().schema, args)?;
    state(dir, |s| Ok(s.report(size(dir)?)))
}
fn state<T>(
    dir: &Path,
    run: impl FnOnce(&mut RecoveryReservations) -> Result<T, String>,
) -> Result<T, String> {
    let mut states = STATES
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| "journal reservation state poisoned")?;
    run(states.entry(dir.into()).or_default())
}
fn size(dir: &Path) -> Result<u64, String> {
    match fs::metadata(dir.join("journal.jsonl")) {
        Ok(m) if m.is_file() => Ok(m.len()),
        Ok(_) => Err("write journal is not a regular file".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(e.to_string()),
    }
}
pub(crate) fn check(dir: &Path, incoming: usize) -> Result<(), String> {
    state(dir, |s| s.check(size(dir)?, incoming, 0, None))
}
pub(crate) fn reserve(dir: &Path, owner: &str, bytes: usize, forward: usize) -> Result<(), String> {
    state(dir, |s| s.reserve(size(dir)?, owner, bytes, forward))
}
pub(crate) fn recovery(dir: &Path, owner: &str, bytes: usize, dry: bool) -> Result<(), String> {
    state(dir, |s| {
        let current = size(dir)?;
        if dry {
            s.clone().ensure(current, owner, bytes)
        } else {
            s.ensure(current, owner, bytes)
        }
    })
}
pub(crate) fn release(dir: &Path, owner: &str) -> Result<(), String> {
    state(dir, |s| {
        s.release(owner);
        Ok(())
    })
}
pub(crate) fn reset(dir: &Path) -> Result<(), String> {
    state(dir, |s| {
        *s = RecoveryReservations::default();
        Ok(())
    })
}
pub(crate) fn append(
    dir: &Path,
    bytes: &[u8],
    outcome: usize,
    owner: Option<&str>,
) -> Result<(), String> {
    append_with(dir, bytes, outcome, owner, |file, bytes| {
        file.write_all(bytes).and_then(|()| file.sync_all())
    })
}
fn append_with(
    dir: &Path,
    bytes: &[u8],
    outcome: usize,
    owner: Option<&str>,
    durable: impl FnOnce(&mut fs::File, &[u8]) -> std::io::Result<()>,
) -> Result<(), String> {
    state(dir, |s| {
        s.check(size(dir)?, bytes.len(), outcome, owner)?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("journal.jsonl"))
            .map_err(|e| e.to_string())?;
        if !f.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("write journal is not a regular file".into());
        }
        s.check(
            f.metadata().map_err(|e| e.to_string())?.len(),
            bytes.len(),
            outcome,
            owner,
        )?;
        durable(&mut f, bytes).map_err(|e| format!("journal durability failed: {e}"))?;
        s.synced(owner, bytes.len());
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn durability_failure(written: usize) {
        let d = dir();
        reserve(&d, "a", 100, 0).unwrap();
        reserve(&d, "b", 100, 0).unwrap();
        let mut file = fs::File::create(d.join("journal.jsonl")).unwrap();
        file.write_all(b"original-prefix\n").unwrap();
        let initial_size = policy::construction::JOURNAL_MAX_BYTES as u64 - 200;
        file.set_len(initial_size).unwrap();
        file.sync_all().unwrap();
        drop(file);
        let before = fs::read(d.join("journal.jsonl")).unwrap();
        let error = append_with(&d, &[b'a'; 100], 0, Some("a"), |file, bytes| {
            file.write_all(&bytes[..written])?;
            Err(std::io::Error::other("injected durability failure"))
        })
        .unwrap_err();
        assert_eq!(
            error,
            "journal durability failed: injected durability failure"
        );
        let mut expected = before;
        expected.extend_from_slice(&vec![b'a'; written]);
        assert_eq!(fs::read(d.join("journal.jsonl")).unwrap(), expected);
        let observation = report(&d, &serde_json::Map::new()).unwrap();
        assert_eq!(observation["journal_bytes"], initial_size + written as u64);
        assert_eq!(observation["recovery_reservation_count"], 2);
        assert_eq!(observation["retained_recovery_bytes"], 200);
        assert_eq!(observation["over_capacity"], true);
        assert_eq!(observation["unreserved_bytes"], 0);
        assert_eq!(observation["write_authorized"], false);
        assert!(check(&d, 1).is_err());
        assert!(recovery(&d, "a", 100, false).is_err());
        assert!(recovery(&d, "b", 100, false).is_err());
        assert!(append(&d, b"x", 0, None).is_err());
        assert!(append(&d, b"x", 0, Some("a")).is_err());
        assert_eq!(report(&d, &serde_json::Map::new()).unwrap(), observation);
        assert_eq!(fs::read(d.join("journal.jsonl")).unwrap(), expected);
        reset(&d).unwrap();
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn partial_write_error_preserves_all_credit_and_actual_prefix_before_refusal() {
        durability_failure(1);
    }
    #[test]
    fn sync_error_after_full_write_preserves_all_credit_and_refuses_further_appends() {
        durability_failure(100);
    }
    #[test]
    fn real_windows_write_lock_preserves_bytes_and_both_recovery_owners() {
        let d = dir();
        reserve(&d, "a", 100, 0).unwrap();
        reserve(&d, "b", 100, 0).unwrap();
        let path = d.join("journal.jsonl");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"independent-prefix\n").unwrap();
        let initial = policy::construction::JOURNAL_MAX_BYTES as u64 - 200;
        file.set_len(initial).unwrap();
        file.sync_all().unwrap();
        drop(file);
        let before = fs::read(&path).unwrap();
        let observation = report(&d, &serde_json::Map::new()).unwrap();
        assert_eq!(observation["retained_recovery_bytes"], 200);
        assert_eq!(observation["recovery_reservation_count"], 2);
        assert_eq!(observation["unreserved_bytes"], 0);
        assert_eq!(observation["over_capacity"], false);
        assert_eq!(observation["write_authorized"], false);

        let locked = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        locked.try_lock().unwrap();
        // A distinct append handle opens successfully; the kernel refuses its write.
        let mut independent = OpenOptions::new().append(true).open(&path).unwrap();
        let write_error = independent.write_all(b"x").unwrap_err();
        assert_eq!(write_error.raw_os_error(), Some(33)); // ERROR_LOCK_VIOLATION
        assert_eq!(independent.metadata().unwrap().len(), initial);
        drop(independent);
        assert_eq!(
            append(&d, &[b'a'; 75], 0, Some("a")).unwrap_err(),
            format!("journal durability failed: {write_error}")
        );
        assert_eq!(report(&d, &serde_json::Map::new()).unwrap(), observation);
        locked.unlock().unwrap();
        drop(locked);
        assert_eq!(fs::read(&path).unwrap(), before);

        // Deliberate adapter calls after unlocking spend only the named owner's credit.
        append(&d, &[b'a'; 75], 0, Some("a")).unwrap();
        assert_eq!(
            report(&d, &serde_json::Map::new()).unwrap()["retained_recovery_bytes"],
            125
        );
        recovery(&d, "b", 100, true).unwrap();
        assert!(recovery(&d, "a", 100, true).is_err());
        assert!(append(&d, b"x", 0, None).is_err());
        append(&d, &[b'a'; 25], 0, Some("a")).unwrap();
        append(&d, &[b'b'; 100], 0, Some("b")).unwrap();
        let mut expected = before;
        expected.extend_from_slice(&[b'a'; 100]);
        expected.extend_from_slice(&[b'b'; 100]);
        assert_eq!(fs::read(&path).unwrap(), expected);
        assert_eq!(expected.len(), policy::construction::JOURNAL_MAX_BYTES);
        assert_eq!(
            report(&d, &serde_json::Map::new()).unwrap()["retained_recovery_bytes"],
            0
        );
        reset(&d).unwrap();
        assert!(
            d.canonicalize().unwrap().starts_with(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .canonicalize()
                    .unwrap()
            )
        );
        fs::remove_file(path).unwrap();
        fs::remove_dir(d).unwrap();
    }
    #[test]
    fn capacity_observes_synced_bytes_without_disclosing_or_changing_reservations() {
        let d = dir();
        reserve(&d, "private_owner", 100, 0).unwrap();
        append(&d, b"test\n", 0, Some("private_owner")).unwrap();
        let before = fs::read(d.join("journal.jsonl")).unwrap();
        let observation = report(&d, &serde_json::Map::new()).unwrap();
        assert_eq!(observation["journal_bytes"], 5);
        assert_eq!(observation["retained_recovery_bytes"], 95);
        assert_eq!(observation["recovery_reservation_count"], 1);
        assert!(!observation.to_string().contains("private_owner"));
        assert_eq!(report(&d, &serde_json::Map::new()).unwrap(), observation);
        assert_eq!(fs::read(d.join("journal.jsonl")).unwrap(), before);
        reset(&d).unwrap();
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn capacity_report_is_read_only_and_rejects_undeclared_arguments() {
        let d = dir();
        let report = report(&d, &serde_json::Map::new()).unwrap();
        assert_eq!(report["journal_bytes"], 0);
        assert_eq!(
            report["unreserved_bytes"],
            policy::construction::JOURNAL_MAX_BYTES
        );
        assert_eq!(report["recovery_reservation_count"], 0);
        assert!(!d.join("journal.jsonl").exists());
        let bad = serde_json::json!({"confirm":"server-write"});
        assert!(super::report(&d, bad.as_object().unwrap()).is_err());
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn dry_recovery_does_not_top_up_and_concurrent_appends_cannot_overspend() {
        let d = dir();
        reserve(&d, "a", 100, 0).unwrap();
        append(&d, &[b'x'; 50], 0, Some("a")).unwrap();
        recovery(&d, "a", 100, true).unwrap();
        let file = fs::OpenOptions::new()
            .write(true)
            .open(d.join("journal.jsonl"))
            .unwrap();
        file.set_len(policy::construction::JOURNAL_MAX_BYTES as u64 - 101)
            .unwrap();
        drop(file);
        append(&d, &[b'x'; 50], 0, None).unwrap();
        assert!(recovery(&d, "a", 100, false).is_err());
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let d = d.clone();
                std::thread::spawn(move || append(&d, b"x", 0, None).is_ok())
            })
            .collect();
        assert_eq!(
            handles
                .into_iter()
                .map(|h| usize::from(h.join().unwrap()))
                .sum::<usize>(),
            1
        );
        reset(&d).unwrap();
        fs::remove_dir_all(d).unwrap();
    }
    use std::sync::atomic::{AtomicU64, Ordering};
    fn dir() -> PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!(
                "journal-budget-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&p).unwrap();
        p
    }
    #[test]
    fn generic_writes_preserve_recovery_and_only_synced_owner_appends_spend_it() {
        let d = dir();
        reserve(&d, "a", 100, 0).unwrap();
        let f = fs::File::create(d.join("journal.jsonl")).unwrap();
        f.set_len(policy::construction::JOURNAL_MAX_BYTES as u64 - 100)
            .unwrap();
        drop(f);
        assert!(append(&d, b"x", 0, Some("unknown")).is_err());
        append(&d, b"x", 0, Some("a")).unwrap();
        assert!(append(&d, b"x", 0, None).is_err());
        release(&d, "a").unwrap();
        append(&d, b"x", 0, None).unwrap();
        reset(&d).unwrap();
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn failed_open_keeps_credit_and_reset_is_directory_scoped() {
        use std::os::windows::fs::OpenOptionsExt;
        let d = dir();
        let other = dir();
        reserve(&d, "a", 100, 0).unwrap();
        reserve(&other, "b", 100, 0).unwrap();
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .share_mode(0)
            .open(d.join("journal.jsonl"))
            .unwrap();
        f.set_len(policy::construction::JOURNAL_MAX_BYTES as u64 - 100)
            .unwrap();
        assert!(append(&d, b"x", 0, Some("a")).is_err());
        drop(f);
        assert!(append(&d, b"x", 0, None).is_err());
        append(&d, b"x", 0, Some("a")).unwrap();
        reset(&other).unwrap();
        assert!(append(&d, b"x", 0, None).is_err());
        reset(&d).unwrap();
        append(&d, b"x", 0, None).unwrap();
        fs::remove_dir_all(d).unwrap();
        fs::remove_dir_all(other).unwrap();
    }
}

#[cfg(test)]
#[path = "journal_process_tests.rs"]
mod process_tests;
