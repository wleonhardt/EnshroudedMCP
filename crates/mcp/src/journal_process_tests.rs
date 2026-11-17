//! Owned subprocess interruption at controlled pre-sync journal boundaries.
use super::*;
use serde_json::{Map, Value, json};
use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const TEST: &str =
    "journal::process_tests::forced_termination_preserves_inflight_bytes_and_session_scope";
const PHASE: &str = "ENSHROUDED_MCP_TEST_JOURNAL_TERMINATION_PHASE";
const DIRECTORY: &str = "ENSHROUDED_MCP_TEST_JOURNAL_TERMINATION_DIRECTORY";
const MARKER: &str = "JOURNAL_TERMINATION_READY ";
const PREFIX: &[u8] = b"prefix\n";
const PAYLOAD: &[u8] = &[b'a'; 100];
const TIMEOUT: Duration = Duration::from_secs(10);

fn fixture_root() -> PathBuf {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let target = project.join("target");
    if target.exists() {
        assert!(target.canonicalize().unwrap().starts_with(&project));
    }
    fs::create_dir_all(&target).unwrap();
    let root = target
        .canonicalize()
        .unwrap()
        .join("journal-termination-tests");
    if root.exists() {
        assert!(root.canonicalize().unwrap().starts_with(&project));
    }
    fs::create_dir_all(&root).unwrap();
    root.canonicalize().unwrap()
}

struct Worker {
    child: Child,
    events: mpsc::Receiver<Result<Value, String>>,
    reader: Option<JoinHandle<()>>,
}
impl Worker {
    fn start(dir: &Path, phase: &str) -> Self {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", TEST, "--nocapture"])
            .env(PHASE, phase)
            .env(DIRECTORY, dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW: these synthetic test children need no visible window.
            command.creation_flags(0x0800_0000);
        }
        let mut child = command.spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, events) = mpsc::channel();
        let reader = thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(error) => {
                        let _ = sender.send(Err(error.to_string()));
                        break;
                    }
                };
                if let Some(index) = line.find(MARKER) {
                    let value = serde_json::from_str(&line[index + MARKER.len()..])
                        .map_err(|error| error.to_string());
                    let _ = sender.send(value);
                }
            }
        });
        Self {
            child,
            events,
            reader: Some(reader),
        }
    }
    fn ready(&self) -> Value {
        let value = self
            .events
            .recv_timeout(TIMEOUT)
            .expect("owned worker did not announce boundary")
            .unwrap();
        assert_eq!(value["pid"], self.child.id());
        value
    }
    fn wait(&mut self) -> ExitStatus {
        let until = Instant::now() + TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < until, "owned worker did not exit");
            thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        // Closing stdin also releases a parked callback if controller assertions fail.
        self.child.stdin.take();
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

fn announce(value: Value) {
    println!("{MARKER}{value}");
    std::io::stdout().flush().unwrap();
}
fn worker(phase: &str) {
    let dir = PathBuf::from(std::env::var_os(DIRECTORY).expect("controller directory required"))
        .canonicalize()
        .unwrap();
    assert_eq!(dir.parent(), Some(fixture_root().as_path()));
    if phase == "inspect" {
        let before = report(&dir, &Map::new()).unwrap();
        assert_eq!(before["recovery_reservation_count"], 0);
        assert_eq!(before["retained_recovery_bytes"], 0);
        assert_eq!(before["write_authorized"], false);
        assert_eq!(
            recovery(&dir, "terminated-owner", 200, true).unwrap_err(),
            "unknown recovery reservation"
        );
        assert_eq!(report(&dir, &Map::new()).unwrap(), before);
        announce(json!({"pid":std::process::id(),"phase":phase,"observation":before}));
        return;
    }
    let written = match phase {
        "partial" => 1,
        "full-before-sync" => PAYLOAD.len(),
        _ => panic!("unknown controller phase"),
    };
    reserve(&dir, "terminated-owner", 200, 0).unwrap();
    let before = report(&dir, &Map::new()).unwrap();
    assert_eq!(before["recovery_reservation_count"], 1);
    assert_eq!(before["retained_recovery_bytes"], 200);
    let result = append_with(&dir, PAYLOAD, 0, Some("terminated-owner"), |file, bytes| {
        file.write_all(&bytes[..written])?;
        announce(
            json!({"pid":std::process::id(),"phase":phase,"written":written,"before":before,"sync_called":false}),
        );
        let mut release = [0];
        std::io::stdin().read_exact(&mut release)?;
        Err(std::io::Error::other(
            "controller released an unfinished append",
        ))
    });
    panic!("controller must terminate the parked writer: {result:?}");
}

#[test]
fn forced_termination_preserves_inflight_bytes_and_session_scope() {
    if let Some(phase) = std::env::var_os(PHASE) {
        worker(phase.to_str().unwrap());
        return;
    }
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = fixture_root();
    for (phase, written) in [("partial", 1), ("full-before-sync", PAYLOAD.len())] {
        let dir = root.join(format!(
            "owned-{}-{}-{phase}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("preserve any prior fixture instead of overwriting it");
        let dir = dir.canonicalize().unwrap();
        assert_eq!(dir.parent(), Some(root.as_path()));
        let path = dir.join("journal.jsonl");
        let mut seed = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        seed.write_all(PREFIX).unwrap();
        seed.sync_all().unwrap();
        drop(seed);
        let mut writer = Worker::start(&dir, phase);
        let ready = writer.ready();
        assert_eq!(ready["phase"], phase);
        assert_eq!(ready["written"], written);
        assert_eq!(ready["sync_called"], false);
        assert_eq!(ready["before"]["retained_recovery_bytes"], 200);
        let expected = [PREFIX, &PAYLOAD[..written]].concat();
        assert_eq!(fs::read(&path).unwrap(), expected);
        assert!(writer.child.try_wait().unwrap().is_none());
        writer.child.kill().unwrap();
        assert!(!writer.wait().success());
        drop(writer);
        assert_eq!(fs::read(&path).unwrap(), expected);
        let mut inspector = Worker::start(&dir, "inspect");
        let reopened = inspector.ready();
        assert_eq!(reopened["phase"], "inspect");
        assert_eq!(reopened["observation"]["journal_bytes"], expected.len());
        assert!(inspector.wait().success());
        drop(inspector);
        assert_eq!(fs::read(&path).unwrap(), expected);
        eprintln!(
            "JOURNAL_TERMINATION {}",
            json!({"phase":phase,"written_bytes":written,"before":ready["before"],"reopened":reopened["observation"],"forced_exit_observed":true,"exact_bytes_preserved":true,"sync_success_acknowledged":false,"power_loss_tested":false,"game_accessed":false})
        );
        assert_eq!(dir.parent(), Some(root.as_path()));
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
