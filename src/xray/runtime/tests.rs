use std::process::{Command, Stdio};
use std::time::Duration;

use super::{TerminationOutcome, process_is_running, terminate_process_gracefully};

#[test]
fn invalid_pid_is_not_running() {
    assert!(!process_is_running(0));
    assert!(!process_is_running(-1));
}

#[test]
fn graceful_termination_ignores_invalid_pid() {
    let outcome = terminate_process_gracefully(0, Duration::from_millis(1))
        .expect("invalid pid should not fail");

    assert_eq!(outcome, TerminationOutcome::NotRunning);
}

#[test]
fn graceful_termination_stops_running_process() {
    let mut child = Command::new("sleep")
        .arg("5")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep should spawn");

    let pid = i64::from(child.id());
    assert!(process_is_running(pid));

    let outcome = terminate_process_gracefully(pid, Duration::from_secs(1))
        .expect("termination should succeed");

    let _ = child.wait();
    assert!(matches!(
        outcome,
        TerminationOutcome::Terminated | TerminationOutcome::Killed
    ));
    assert!(!process_is_running(pid));
}
