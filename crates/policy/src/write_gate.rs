//! The gate in front of every tool that changes the world, ported from
//! ValheimMCP's `WriteGate`: a flag file the owner creates by hand (and that
//! goes stale), plus a confirm string on every call. A dry run skips the gate
//! only when the tool really only reports.

use std::time::Duration;

/// What every write call must pass as `confirm`.
pub const CONFIRM: &str = "server-write";
/// The flag file's name, in the mod's folder.
pub const FLAG: &str = "ALLOW_WRITES";
/// How long a flag file stays valid after it was last touched.
pub const FLAG_MAX_AGE: Duration = Duration::from_secs(30 * 60);

/// Returns the age of a regular flag file. Missing, unreadable and future-dated
/// files fail closed. Freshness remains the responsibility of [`check`].
#[must_use]
pub fn flag_age(path: &std::path::Path) -> Option<Duration> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    metadata.modified().ok()?.elapsed().ok()
}

/// `Ok` when the call may change the world. `flag_age` is the flag file's age,
/// `None` when it does not exist. `honours_dry` says the tool's dry run only
/// reports; a tool that ignored `dry` would otherwise write with the gate shut.
///
/// # Errors
/// The refusal, worded for the caller.
pub fn check(
    dry: bool,
    honours_dry: bool,
    flag_age: Option<Duration>,
    confirm: Option<&str>,
) -> Result<(), String> {
    if dry && honours_dry {
        return Ok(());
    }
    let Some(age) = flag_age else {
        return Err(format!(
            "writes are off: create {FLAG} in the mod's folder (and remove it when done)"
        ));
    };
    if age > FLAG_MAX_AGE {
        return Err(format!(
            "the write flag is older than {} min: touch {FLAG} again",
            FLAG_MAX_AGE.as_secs() / 60
        ));
    }
    if confirm != Some(CONFIRM) {
        return Err(format!("pass confirm: \"{CONFIRM}\" with every write"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_metadata_requires_regular_nonfuture_file_and_preserves_staleness() {
        let directory = std::env::temp_dir().join(format!(
            "mcp-policy-flag-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join(FLAG);
        assert_eq!(flag_age(&path), None);
        assert_eq!(flag_age(&directory), None);
        let file = std::fs::File::create(&path).unwrap();
        assert!(check(false, true, flag_age(&path), Some(CONFIRM)).is_ok());
        file.set_times(
            std::fs::FileTimes::new()
                .set_modified(std::time::SystemTime::now() + Duration::from_secs(3600)),
        )
        .unwrap();
        assert_eq!(flag_age(&path), None);
        file.set_times(
            std::fs::FileTimes::new().set_modified(
                std::time::SystemTime::now() - FLAG_MAX_AGE - Duration::from_secs(60),
            ),
        )
        .unwrap();
        assert!(check(false, true, flag_age(&path), Some(CONFIRM)).is_err());
        drop(file);
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(&directory).unwrap();
    }

    const FRESH: Option<Duration> = Some(Duration::from_secs(60));

    #[test]
    fn a_fresh_flag_and_the_confirm_string_open_the_gate() {
        assert_eq!(check(false, true, FRESH, Some(CONFIRM)), Ok(()));
    }

    #[test]
    fn without_a_flag_writes_are_off() {
        let err = check(false, true, None, Some(CONFIRM)).unwrap_err();
        assert!(err.starts_with("writes are off"), "{err}");
    }

    #[test]
    fn a_stale_flag_is_refused() {
        let old = Some(FLAG_MAX_AGE + Duration::from_secs(1));
        assert!(
            check(false, true, old, Some(CONFIRM))
                .unwrap_err()
                .contains("older than 30 min")
        );
    }

    #[test]
    fn the_confirm_string_is_required() {
        assert!(check(false, true, FRESH, None).is_err());
        assert!(check(false, true, FRESH, Some("yes")).is_err());
    }

    #[test]
    fn a_dry_run_skips_the_gate_only_when_the_tool_honours_it() {
        assert_eq!(check(true, true, None, None), Ok(()));
        assert!(check(true, false, None, None).is_err());
    }
}
