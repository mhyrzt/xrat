use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::process_is_running;

const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminationOutcome {
    NotRunning,
    Terminated,
    Killed,
}

pub fn terminate_process(pid: i64) -> Result<bool, crate::app::AppError> {
    if pid <= 0 {
        return Ok(false);
    }

    #[cfg(unix)]
    {
        let status = Command::new("kill").arg(pid.to_string()).status()?;
        Ok(status.success())
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        Ok(false)
    }
}

pub fn terminate_process_gracefully(
    pid: i64,
    timeout: Duration,
) -> Result<TerminationOutcome, crate::app::AppError> {
    if !process_is_running(pid) {
        return Ok(TerminationOutcome::NotRunning);
    }

    if !send_signal(pid, "TERM")? {
        return Ok(TerminationOutcome::NotRunning);
    }

    let start = Instant::now();
    while start.elapsed() < timeout {
        if !process_is_running(pid) {
            return Ok(TerminationOutcome::Terminated);
        }
        std::thread::sleep(PROCESS_POLL_INTERVAL);
    }

    if process_is_running(pid) {
        let _ = send_signal(pid, "KILL")?;
        return Ok(TerminationOutcome::Killed);
    }

    Ok(TerminationOutcome::Terminated)
}

fn send_signal(pid: i64, signal: &str) -> Result<bool, crate::app::AppError> {
    if pid <= 0 {
        return Ok(false);
    }

    #[cfg(unix)]
    {
        let status = Command::new("kill")
            .arg(format!("-{signal}"))
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        Ok(status.success())
    }

    #[cfg(not(unix))]
    {
        let _ = (pid, signal);
        Ok(false)
    }
}
