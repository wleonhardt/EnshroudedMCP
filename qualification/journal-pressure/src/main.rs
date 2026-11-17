//! Disposable filesystem qualification using the unchanged production adapter.
#[path = "../../../crates/mcp/src/journal.rs"]
mod journal;

use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::windows::fs::OpenOptionsExt,
    path::Path,
    sync::{Arc, Barrier},
};

fn observation(dir: &Path) -> serde_json::Value {
    journal::report(dir, &serde_json::Map::new()).unwrap()
}

fn main() {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let root = project.join("gamedata/journal-pressure-qualification");
    fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    assert!(root.starts_with(&project));
    let mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let output = root.join(format!("report-{mode}.json"));
    assert!(!output.exists(), "inspect previous report; never overwrite");
    let dir = root.join("fixture");
    assert!(
        !dir.exists(),
        "inspect previous fixture; never reuse implicitly"
    );
    fs::create_dir(&dir).unwrap();
    let limit = policy::construction::JOURNAL_MAX_BYTES as u64;
    journal::reserve(&dir, "owner_a", 100, 0).unwrap();
    journal::reserve(&dir, "owner_b", 100, 0).unwrap();
    let mut file = fs::File::create(dir.join("journal.jsonl")).unwrap();
    file.write_all(b"independent-prefix\n").unwrap();
    file.set_len(limit - 264).unwrap();
    file.sync_all().unwrap();
    drop(file);
    let initial = fs::read(dir.join("journal.jsonl")).unwrap();
    assert_eq!(observation(&dir)["unreserved_bytes"], 64);

    let barrier = Arc::new(Barrier::new(129));
    let threads: Vec<_> = (0..128)
        .map(|_| {
            let barrier = barrier.clone();
            let dir = dir.clone();
            std::thread::spawn(move || {
                barrier.wait();
                journal::append(&dir, b"x", 0, None).is_ok()
            })
        })
        .collect();
    barrier.wait();
    let admitted: usize = threads
        .into_iter()
        .map(|t| usize::from(t.join().unwrap()))
        .sum();
    assert_eq!(admitted, 64);
    let after_pressure = fs::read(dir.join("journal.jsonl")).unwrap();
    assert_eq!(&after_pressure[..initial.len()], initial);
    assert_eq!(&after_pressure[initial.len()..], &[b'x'; 64]);
    assert_eq!(observation(&dir)["retained_recovery_bytes"], 200);
    assert!(journal::check(&dir, 1).is_err());
    assert!(journal::append(&dir, b"x", 0, Some("unknown")).is_err());
    journal::recovery(&dir, "owner_a", 100, true).unwrap();
    journal::recovery(&dir, "owner_a", 100, false).unwrap();

    let locked = OpenOptions::new()
        .write(true)
        .share_mode(0)
        .open(dir.join("journal.jsonl"))
        .unwrap();
    assert!(journal::append(&dir, &[b'a'; 75], 0, Some("owner_a")).is_err());
    drop(locked);
    assert_eq!(observation(&dir)["retained_recovery_bytes"], 200);
    assert_eq!(fs::read(dir.join("journal.jsonl")).unwrap(), after_pressure);
    journal::append(&dir, &[b'a'; 75], 0, Some("owner_a")).unwrap();
    assert_eq!(observation(&dir)["retained_recovery_bytes"], 125);
    assert!(journal::recovery(&dir, "owner_a", 100, false).is_err());
    assert!(journal::append(&dir, b"x", 0, None).is_err());
    journal::release(&dir, "owner_a").unwrap();
    assert_eq!(observation(&dir)["unreserved_bytes"], 25);
    journal::append(&dir, &[b'g'; 25], 0, None).unwrap();
    assert!(journal::append(&dir, b"x", 0, None).is_err());
    journal::append(&dir, &[b'b'; 100], 0, Some("owner_b")).unwrap();
    let final_bytes = fs::read(dir.join("journal.jsonl")).unwrap();
    assert_eq!(final_bytes.len() as u64, limit);
    assert_eq!(&final_bytes[..initial.len()], initial);
    assert_eq!(&final_bytes[initial.len()..initial.len() + 64], &[b'x'; 64]);
    assert_eq!(
        &final_bytes[initial.len() + 64..initial.len() + 139],
        &[b'a'; 75]
    );
    assert_eq!(
        &final_bytes[initial.len() + 139..initial.len() + 164],
        &[b'g'; 25]
    );
    assert_eq!(&final_bytes[initial.len() + 164..], &[b'b'; 100]);
    journal::release(&dir, "owner_b").unwrap();
    assert_eq!(observation(&dir)["recovery_reservation_count"], 0);
    journal::reset(&dir).unwrap();
    let result = serde_json::json!({"scope":"disposable filesystem; no game access",
        "writers":128,"admitted":admitted,"refused":128-admitted,
        "prefix_preserved":true,"lock_refusal_credit_preserved":true,
        "released_credit_reusable":true,"remaining_owner_recovery_succeeds":true,
        "exact_final_bytes":limit,"partial_write_or_sync_fault_tested":false});
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    assert!(dir.canonicalize().unwrap().starts_with(&root));
    // Delete only the two explicitly named disposable paths, never recursively.
    fs::remove_file(dir.join("journal.jsonl")).unwrap();
    fs::remove_dir(&dir).unwrap();
    println!("{result}");
}
