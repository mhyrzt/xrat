use std::time::Duration;

use tokio::sync::mpsc;

const REPO: &str = "mhyrzt/xrat";
const TIMEOUT_SECS: u64 = 8;

/// Spawn a background check for a newer release. When a strictly newer tag is
/// published, it is sent through `tx`. Every failure (offline, parse error,
/// rate limit) is swallowed so the TUI never surfaces a check problem.
pub fn spawn_version_check(tx: mpsc::UnboundedSender<String>) {
    tokio::spawn(async move {
        if let Some(tag) = newer_release_tag().await {
            let _ = tx.send(tag);
        }
    });
}

async fn newer_release_tag() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .user_agent(concat!("xrat/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body = client
        .get(&url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .await
        .ok()?;
    let payload: serde_json::Value = serde_json::from_str(&body).ok()?;
    let tag = payload.get("tag_name")?.as_str()?.to_string();
    is_newer(&tag, env!("CARGO_PKG_VERSION")).then_some(tag)
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_semver(latest), parse_semver(current)) {
        (Some(latest), Some(current)) => latest > current,
        _ => normalize(latest) != normalize(current),
    }
}

fn normalize(tag: &str) -> &str {
    tag.trim().trim_start_matches('v')
}

fn parse_semver(tag: &str) -> Option<(u32, u32, u32)> {
    let mut parts = normalize(tag).split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn detects_newer_release() {
        assert!(is_newer("v0.3.0", "0.2.1"));
        assert!(is_newer("0.2.2", "0.2.1"));
        assert!(is_newer("1.0.0", "0.9.9"));
    }

    #[test]
    fn ignores_same_or_older() {
        assert!(!is_newer("v0.2.1", "0.2.1"));
        assert!(!is_newer("0.2.0", "0.2.1"));
        assert!(!is_newer("v0.1.0", "0.2.1"));
    }
}
