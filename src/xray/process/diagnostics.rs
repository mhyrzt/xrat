//! Condenses noisy Xray stdout/stderr into a single root-cause line.
//!
//! Xray prints a startup banner, timestamped info logs, the temp config path,
//! and a long ` > `-separated error chain. Only the deepest segment of that
//! chain is actionable, so callers (probe failures, daemon startup errors)
//! should surface that instead of the raw dump.

pub(super) fn summarize_xray_failure(raw: &str) -> String {
    if let Some(cause) = deepest_error_cause(raw) {
        return cause;
    }

    let meaningful: Vec<&str> = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !is_noise(line))
        .collect();

    if meaningful.is_empty() {
        "Xray exited without reporting an error".to_string()
    } else {
        meaningful.join("; ")
    }
}

fn deepest_error_cause(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.to_ascii_lowercase().contains("fail") {
            continue;
        }
        let segment = trimmed.rsplit(" > ").next().unwrap_or(trimmed).trim();
        let cause = strip_package_prefixes(segment).trim_end_matches('.').trim();
        if !cause.is_empty() {
            return Some(cause.to_string());
        }
    }
    None
}

/// Drops leading Go package labels like `infra/conf: ` while keeping
/// meaningful prefixes such as `REALITY:` that have no path separator.
fn strip_package_prefixes(segment: &str) -> &str {
    let mut rest = segment;
    while let Some((label, tail)) = rest.split_once(": ") {
        if label.contains('/') && !label.contains(' ') {
            rest = tail.trim_start();
        } else {
            break;
        }
    }
    rest
}

fn is_noise(line: &str) -> bool {
    if line == "A unified platform for anti-censorship." {
        return true;
    }

    let lower = line.to_ascii_lowercase();
    if line.starts_with("Xray ") && lower.contains("penetrates everything") {
        return true;
    }

    is_timestamped_log(line)
}

fn is_timestamped_log(line: &str) -> bool {
    let bytes = line.as_bytes();
    let has_date_prefix =
        bytes.len() > 10 && bytes[..4].iter().all(u8::is_ascii_digit) && bytes[4] == b'/';
    has_date_prefix
        && (line.contains("[Info]") || line.contains("[Debug]") || line.contains("[Warning]"))
}

#[cfg(test)]
mod tests {
    use super::summarize_xray_failure;

    #[test]
    fn extracts_reality_root_cause_from_full_dump() {
        let raw = concat!(
            "A unified platform for anti-censorship.\n",
            "2026/06/20 07:48:33.727847 [Info] infra/conf/serial: Reading config: ",
            "&{Name:/tmp/.tmprvTxEI.json Format:json}\n",
            "Failed to start: main: failed to load config files: [/tmp/.tmprvTxEI.json] > ",
            "infra/conf: failed to build outbound config with tag proxy > ",
            "infra/conf: failed to build stream settings for outbound detour > ",
            "infra/conf: REALITY: Empty \"realitySettings\".",
        );

        assert_eq!(
            summarize_xray_failure(raw),
            "REALITY: Empty \"realitySettings\""
        );
    }

    #[test]
    fn drops_banner_and_version_when_no_error_chain() {
        let raw = concat!(
            "A unified platform for anti-censorship.\n",
            "Xray 26.3.27 (Xray, Penetrates Everything.) d2758a0 (go1.26.1 linux/amd64)",
        );

        assert_eq!(
            summarize_xray_failure(raw),
            "Xray exited without reporting an error"
        );
    }

    #[test]
    fn keeps_meaningful_line_without_chain() {
        let raw = concat!(
            "A unified platform for anti-censorship.\n",
            "2026/06/20 07:48:33 [Warning] core: noisy warning\n",
            "permission denied binding port 1080",
        );

        assert_eq!(
            summarize_xray_failure(raw),
            "permission denied binding port 1080"
        );
    }
}
