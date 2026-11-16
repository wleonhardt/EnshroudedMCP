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
        f.write_all(bytes)
            .and_then(|()| f.sync_all())
            .map_err(|e| format!("journal durability failed: {e}"))?;
        s.synced(owner, bytes.len());
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(
            crate::tools::GameTools::journal(Some(&d), "test", &serde_json::json!({}), 0).is_err()
        );
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
