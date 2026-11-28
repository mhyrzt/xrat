use super::{Check, check};

pub(super) async fn inspect_runtime_owner(runtime_dir: Option<&std::path::Path>) -> Check {
    #[cfg(target_os = "linux")]
    {
        let Some(runtime_dir) = runtime_dir else {
            return check(
                "runtime_owner",
                "unknown",
                "Runtime directory could not be determined; daemon privileges are unknown.",
            );
        };
        let socket = crate::app::daemon::ipc::default_socket_path(runtime_dir);
        let stream = match tokio::time::timeout(
            std::time::Duration::from_secs(1),
            tokio::net::UnixStream::connect(&socket),
        )
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(error)) => {
                return check(
                    "runtime_owner",
                    "unknown",
                    format!(
                        "Cannot connect to daemon socket {}: {error}; the gateway runtime owner is unknown.",
                        socket.display()
                    ),
                );
            }
            Err(_) => {
                return check(
                    "runtime_owner",
                    "unknown",
                    "Timed out connecting to the daemon socket; runtime owner privileges are unknown.",
                );
            }
        };
        let Some(pid) = stream
            .peer_cred()
            .ok()
            .and_then(|cred| cred.pid())
            .and_then(|pid| u32::try_from(pid).ok())
        else {
            return check(
                "runtime_owner",
                "unknown",
                "Cannot read daemon socket peer credentials; runtime owner privileges are unknown.",
            );
        };
        match std::fs::read_to_string(format!("/proc/{pid}/status")) {
            Ok(status) => {
                let observed = process_privileges(&status, "This socket peer");
                check(
                    "runtime_owner",
                    observed.state,
                    format!(
                        "Socket peer PID {pid}: {} Peer identity and engine capability inheritance are not verified.",
                        observed.detail
                    ),
                )
            }
            Err(error) => check(
                "runtime_owner",
                "unknown",
                format!("Cannot inspect daemon socket peer PID {pid}: {error}"),
            ),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = runtime_dir;
        check(
            "runtime_owner",
            "unknown",
            "Gateway runtime owner inspection requires Linux.",
        )
    }
}

pub(super) fn caller_privileges(status: &str) -> Check {
    process_privileges(status, "This CLI process")
}

pub(super) fn process_privileges(status: &str, subject: &str) -> Check {
    let field = |name: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .map(str::trim)
    };
    let capability = field("CapEff:").and_then(|raw| u64::from_str_radix(raw, 16).ok());
    let no_new_privs = field("NoNewPrivs:").and_then(|raw| match raw {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    });
    match (capability, no_new_privs) {
        (Some(mask), Some(no_new_privs)) => check(
            "caller_privileges",
            "incomplete",
            format!(
                "{subject} {} effective CAP_NET_ADMIN; NoNewPrivs={}. Capability inheritance is not verified.",
                if mask & (1 << 12) != 0 {
                    "has"
                } else {
                    "does not have"
                },
                u8::from(no_new_privs)
            ),
        ),
        _ => check(
            "caller_privileges",
            "unknown",
            "Cannot parse effective capabilities and NoNewPrivs for the CLI process.",
        ),
    }
}
