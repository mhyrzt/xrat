use super::*;
use crate::model::Node;
use crate::model::Protocol;
use crate::prober::FailureKind;
use crate::prober::real_delay::check::find_available_port;
use crate::prober::real_delay::check::request::make_proxied_request;
use std::path::Path;
use std::time::Duration;

#[tokio::test]
async fn test_find_available_port() {
    let port = match find_available_port().await {
        Ok(port) => port,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => return,
        Err(error) => panic!("unexpected error: {error}"),
    };
    assert!(port > 0);
}

#[tokio::test]
async fn test_real_delay_check_invalid_config() {
    // Test with an invalid node (missing required fields)
    let node = Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: None, // Missing required UUID
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

    let result = real_delay_check(
        &node,
        crate::app::config::defaults::DEFAULT_REAL_DELAY_TEST_URL,
        Path::new("xray"),
        Duration::from_secs(5),
        Duration::from_secs(10),
        &crate::xray::XrayGenOptions::default(),
    )
    .await;

    assert!(!result.success);
    assert!(matches!(result.failure_kind, Some(FailureKind::Process)));
}

#[tokio::test]
async fn test_make_proxied_request_handles_proxy_errors_gracefully() {
    let result = make_proxied_request(
        0,
        "https://www.gstatic.com/generate_204",
        Duration::from_secs(2),
    )
    .await;

    assert!(!result.success);
    assert!(result.failure_kind.is_some());
    assert!(result.failure_reason.is_some());
}
