use policy::game_reply;
use std::{sync::mpsc, time::Duration};

#[test]
fn sender_loss_is_not_an_elapsed_deadline_or_execution_verdict() {
    let (tx, rx) = mpsc::sync_channel::<()>(1);
    drop(tx);
    let reason = game_reply::receive(&rx, Duration::ZERO).unwrap_err();
    assert!(reason.contains("disconnected"), "{reason}");
    assert!(!reason.contains("deadline elapsed"), "{reason}");
    assert!(reason.contains("execution outcome unknown"), "{reason}");
    assert!(reason.contains("status/journal"), "{reason}");
}

#[test]
fn live_producer_without_a_reply_is_a_deadline_with_unknown_execution() {
    let (tx, rx) = mpsc::sync_channel::<()>(1);
    let reason = game_reply::receive(&rx, Duration::ZERO).unwrap_err();
    assert!(reason.contains("deadline elapsed"), "{reason}");
    assert!(!reason.contains("disconnected"), "{reason}");
    assert!(reason.contains("execution outcome unknown"), "{reason}");
    assert!(reason.contains("status/journal"), "{reason}");
    drop(tx);
}

#[test]
fn known_game_reply_and_game_refusal_are_returned_unchanged() {
    let (tx, rx) = mpsc::sync_channel(1);
    tx.send(Ok::<_, String>(7)).unwrap();
    assert_eq!(game_reply::receive(&rx, Duration::ZERO), Ok(Ok(7)));
    tx.send(Err::<u32, _>("retained game refusal".to_owned()))
        .unwrap();
    drop(tx);
    assert_eq!(
        game_reply::receive(&rx, Duration::ZERO),
        Ok(Err("retained game refusal".to_owned()))
    );
}

#[test]
fn deadline_does_not_cancel_late_producer_or_replay_it() {
    let (tx, rx) = mpsc::sync_channel(1);
    let reason = game_reply::receive(&rx, Duration::ZERO).unwrap_err();
    drop(rx);
    let mut executions = 0;
    let mut queued = || {
        executions += 1;
        tx.send(42)
    };
    assert!(queued().is_err());
    assert_eq!(executions, 1);
    assert!(reason.contains("execution outcome unknown"), "{reason}");
}
