use super::*;
use crate::model::Node;
use crate::model::Protocol;
use crate::tester::FailureKind;
use crate::tester::download::check::calculate_mbps;
use std::path::Path;
use std::time::Duration;

#[test]
fn calculates_download_mbps() {
    let speed = calculate_mbps(1_000_000, Duration::from_secs(1));
    assert_eq!(speed, 8.0);
}

#[tokio::test]
async fn download_speed_check_rejects_invalid_config() {
    let node = Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: None,
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        name: Some("test".to_string()),
        extensions: None,
        raw_config: "".to_string(),
    };

    let result = download_speed_check(
        &node,
        crate::app::config::defaults::DEFAULT_DOWNLOAD_TEST_URL,
        Path::new("xray"),
        Duration::from_secs(5),
        Duration::from_secs(10),
    )
    .await;

    assert!(!result.success);
    assert!(matches!(result.failure_kind, Some(FailureKind::Process)));
}
