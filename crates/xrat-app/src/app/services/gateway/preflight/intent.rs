use super::{Check, check};

pub(super) fn inspect_gateway_intent(runtime_dir: Option<&std::path::Path>) -> Vec<Check> {
    let intent = match runtime_dir.map(super::super::journal::load) {
        Some(Ok(Some(intent))) => intent,
        Some(Ok(None)) => {
            return vec![check(
                "installation_intent",
                "ok",
                "No gateway installation intent found.",
            )];
        }
        Some(Err(error)) => {
            return vec![check(
                "installation_intent",
                "unknown",
                format!("Cannot read gateway installation intent: {error}"),
            )];
        }
        None => {
            return vec![check(
                "installation_intent",
                "unknown",
                "Runtime directory could not be determined.",
            )];
        }
    };
    let guard = match super::super::firewall::inspect_closed(&intent) {
        Ok(true) => check(
            "closed_guard",
            "ok",
            "Observed the closed IPv4 LAN forwarding guard.",
        ),
        Ok(false) => {
            #[cfg(target_os = "linux")]
            if let Some(directory) = runtime_dir {
                match super::super::lifecycle::inspect_active(directory, &intent) {
                    Ok(true) => {
                        return vec![
                            check(
                                "installation_intent",
                                "ok",
                                format!(
                                    "Found installation intent {} for {}.",
                                    intent.installation_id, intent.lan_interface
                                ),
                            ),
                            check(
                                "active_guard",
                                "ok",
                                "Observed the active IPv4 gateway forwarding guard.",
                            ),
                        ];
                    }
                    Ok(false) => {}
                    Err(error) => {
                        return vec![check(
                            "active_guard",
                            "unknown",
                            format!("Cannot verify the active forwarding guard: {error}"),
                        )];
                    }
                }
            }
            check(
                "closed_guard",
                "failed",
                "The expected guard or LAN interface identity is missing or differs from the installation intent.",
            )
        }
        Err(error) => check(
            "closed_guard",
            "unknown",
            format!("Cannot verify the closed forwarding guard: {error}"),
        ),
    };
    vec![
        check(
            "installation_intent",
            "incomplete",
            format!(
                "Found installation intent {} for {}.",
                intent.installation_id, intent.lan_interface
            ),
        ),
        guard,
    ]
}
