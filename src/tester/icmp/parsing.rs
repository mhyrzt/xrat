use super::FailureKind;

pub fn parse_ping_latency(output: &str) -> Option<u32> {
    for line in output.lines() {
        if let Some(time_pos) = line.find("time=") {
            let time_str = &line[time_pos + 5..];
            if let Some(space_pos) = time_str.find(|c: char| c.is_whitespace()) {
                let num_str = &time_str[..space_pos];
                if let Ok(ms) = num_str.parse::<f64>() {
                    return Some(ms.round() as u32);
                }
            }
        }
    }
    None
}

pub fn classify_ping_failure(output: &str) -> (FailureKind, String) {
    let lower = output.to_lowercase();

    if lower.contains("operation not permitted") || lower.contains("permission denied") {
        (
            FailureKind::PermissionDenied,
            "ICMP ping requires additional permissions".to_string(),
        )
    } else if lower.contains("unreachable") || lower.contains("no route") {
        (FailureKind::Unreachable, "Host unreachable".to_string())
    } else if lower.contains("timeout") || lower.contains("timed out") {
        (FailureKind::Timeout, "Ping timeout".to_string())
    } else if lower.contains("unknown host") || lower.contains("cannot resolve") {
        (FailureKind::Dns, "DNS resolution failed".to_string())
    } else {
        (FailureKind::Unknown, "Ping failed".to_string())
    }
}
