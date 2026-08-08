use super::*;
use crate::model::Node;
use crate::model::Protocol;
use crate::prober::FailureKind;
use crate::prober::real_delay::AcceptedHttpStatuses;
use crate::prober::real_delay::check::find_available_port;
use crate::prober::real_delay::check::request::{
    make_proxied_request, make_request, redirect_policy,
};
use axum::Router;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::get;
use reqwest::Client;
use std::path::Path;
use std::time::Duration;

async fn spawn_http_server() -> (String, tokio::task::JoinHandle<()>) {
    let app = Router::new()
        .route("/ok", get(|| async { StatusCode::OK }))
        .route("/forbidden", get(|| async { StatusCode::FORBIDDEN }))
        .route("/redirect", get(|| async { Redirect::temporary("/ok") }))
        .route("/loop-a", get(|| async { Redirect::temporary("/loop-b") }))
        .route("/loop-b", get(|| async { Redirect::temporary("/loop-a") }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let address = listener.local_addr().expect("test server address");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve test routes");
    });
    (format!("http://{address}"), handle)
}

fn test_client(follow_redirects: bool) -> Client {
    Client::builder()
        .redirect(redirect_policy(follow_redirects))
        .build()
        .expect("build test client")
}

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
        &AcceptedHttpStatuses::default(),
        true,
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
        &AcceptedHttpStatuses::default(),
        true,
    )
    .await;

    assert!(!result.success);
    assert!(result.failure_kind.is_some());
    assert!(result.failure_reason.is_some());
}

#[tokio::test]
async fn request_accepts_exact_codes_and_inclusive_ranges() {
    let (base_url, server) = spawn_http_server().await;
    let accepted =
        AcceptedHttpStatuses::new(vec![403], vec![(300, 399)]).expect("accepted statuses");
    let client = test_client(false);

    let forbidden = make_request(&client, &format!("{base_url}/forbidden"), &accepted).await;
    let redirect = make_request(&client, &format!("{base_url}/redirect"), &accepted).await;

    assert!(forbidden.success);
    assert_eq!(forbidden.http_status, Some(403));
    assert!(redirect.success);
    assert_eq!(redirect.http_status, Some(307));
    server.abort();
}

#[tokio::test]
async fn request_status_mismatch_retains_status_and_ttfb() {
    let (base_url, server) = spawn_http_server().await;
    let result = make_request(
        &test_client(false),
        &format!("{base_url}/forbidden"),
        &AcceptedHttpStatuses::default(),
    )
    .await;

    assert!(!result.success);
    assert_eq!(result.http_status, Some(403));
    assert!(result.ttfb_ms.is_some());
    assert!(result.failure_reason.as_deref().is_some_and(|reason| {
        reason.contains(&format!("from {base_url}/forbidden"))
            && reason.contains("expected 200-299")
    }));
    server.abort();
}

#[tokio::test]
async fn redirect_following_checks_terminal_status() {
    let (base_url, server) = spawn_http_server().await;
    let result = make_request(
        &test_client(true),
        &format!("{base_url}/redirect"),
        &AcceptedHttpStatuses::new(vec![200], Vec::new()).expect("accepted statuses"),
    )
    .await;

    assert!(result.success);
    assert_eq!(result.http_status, Some(200));
    server.abort();
}

#[tokio::test]
async fn redirect_loop_fails_at_redirect_limit() {
    let (base_url, server) = spawn_http_server().await;
    let result = make_request(
        &test_client(true),
        &format!("{base_url}/loop-a"),
        &AcceptedHttpStatuses::default(),
    )
    .await;

    assert!(!result.success);
    assert_eq!(result.http_status, None);
    assert!(
        result
            .failure_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("10-hop limit"))
    );
    server.abort();
}
