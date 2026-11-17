//! Real process-boundary qualification of session-local journal reservations.
#[path = "../../../../crates/mcp/src/journal.rs"]
mod journal;

use std::{fs, path::Path, process::Command};

fn observation(dir: &Path) -> serde_json::Value {
    journal::report(dir, &serde_json::Map::new()).unwrap()
}

fn child(phase: &str, dir: &Path) -> serde_json::Value {
    let output = Command::new(std::env::current_exe().unwrap())
        .arg(phase)
        .output()
        .unwrap();
    assert!(output.status.success(), "child failed: {:?}", output.status);
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_ne!(report["pid"], std::process::id());
    assert_eq!(
        fs::read(dir.join("journal.jsonl")).unwrap(),
        b"prefix\nowner-record\n"
    );
    report
}

fn main() {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let root = project.join("gamedata/journal-restart-qualification");
    let dir = root.join(format!("fixture-{mode}"));
    let phase = std::env::args().nth(1);
    if let Some(phase) = phase {
        assert!(dir.canonicalize().unwrap().starts_with(&project));
        let report = match phase.as_str() {
            "seed" => {
                journal::reserve(&dir, "session_owner", 200, 0).unwrap();
                journal::recovery(&dir, "session_owner", 200, true).unwrap();
                journal::check(&dir, 13).unwrap();
                journal::append(&dir, b"owner-record\n", 0, Some("session_owner")).unwrap();
                let report = observation(&dir);
                assert_eq!(report["recovery_reservation_count"], 1);
                assert_eq!(report["retained_recovery_bytes"], 187);
                report
            }
            "reopen" => {
                let report = observation(&dir);
                assert_eq!(report["journal_bytes"], 20);
                assert_eq!(report["recovery_reservation_count"], 0);
                assert_eq!(report["retained_recovery_bytes"], 0);
                journal::check(&dir, 1).unwrap();
                journal::release(&dir, "session_owner").unwrap();
                journal::reset(&dir).unwrap();
                assert_eq!(observation(&dir), report);
                report
            }
            _ => panic!("unknown private qualification phase"),
        };
        println!(
            "{}",
            serde_json::json!({"pid":std::process::id(),"observation":report})
        );
        return;
    }
    fs::create_dir_all(&root).unwrap();
    assert!(root.canonicalize().unwrap().starts_with(&project));
    let output = root.join(format!("report-{mode}.json"));
    assert!(
        !output.exists() && !dir.exists(),
        "inspect prior evidence; never overwrite"
    );
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("journal.jsonl"), b"prefix\n").unwrap();
    let seeded = child("seed", &dir);
    let reopened = child("reopen", &dir);
    // PID reuse is possible; distinct child lifetimes, not numeric inequality,
    // establish the process boundary. Each child is awaited before the next.
    let report = serde_json::json!({"scope":"graceful child-process exit and reopen; no game access",
        "seeded":seeded,"reopened":reopened,"exact_bytes_preserved":true,
        "reservation_state_survived_restart":false,"gameplay_authority_recovered":false,
        "power_loss_or_forced_termination_tested":false,"partial_write_or_sync_fault_tested":false});
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    file.write_all(&serde_json::to_vec_pretty(&report).unwrap())
        .unwrap();
    file.sync_all().unwrap();
    fs::remove_file(dir.join("journal.jsonl")).unwrap();
    fs::remove_dir(&dir).unwrap();
    println!("{report}");
}
