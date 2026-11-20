//! Game-free receipt of one queued game-thread result; never cancels its producer.
use std::{sync::mpsc, time::Duration};

pub fn receive<T>(rx: &mpsc::Receiver<T>, wait: Duration) -> Result<T, String> {
    rx.recv_timeout(wait).map_err(|error| match error {
        mpsc::RecvTimeoutError::Timeout => format!(
            "the game-thread reply deadline elapsed after {} s; execution outcome unknown; inspect retained status/journal before write recovery",
            wait.as_secs()
        ),
        mpsc::RecvTimeoutError::Disconnected => "the game-thread reply channel disconnected before a reply; execution outcome unknown; inspect retained status/journal before write recovery".to_owned(),
    })
}
