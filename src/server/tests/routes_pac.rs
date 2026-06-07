use axum::body::to_bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, header};

use super::multi_config_state;
use crate::server::ServerError;
use crate::server::routes::pac;

#[tokio::test]
async fn pac_route_sets_content_type_and_is_unauthenticated() {
    // An API key is configured, but the PAC route must not require it.
    let state = multi_config_state(Some("secret"), 1).await;

    let response = pac::proxy_pac(State(state), host_headers("127.0.0.1:18203"))
        .await
        .expect("pac route should succeed without auth");

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .expect("content type should be set")
        .to_str()
        .expect("content type should be ascii");
    assert_eq!(content_type, "application/x-ns-proxy-autoconfig");

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let text = String::from_utf8(body.to_vec()).expect("utf8");
    // No running runtime session, so the PAC routes everything DIRECT.
    assert!(text.contains("function FindProxyForURL"));
    assert!(text.contains("return \"DIRECT\";"));
}

#[tokio::test]
async fn pac_route_rejects_unallowed_host_header() {
    let state = multi_config_state(None, 1).await;

    let result = pac::proxy_pac(State(state), host_headers("evil.example")).await;

    assert!(matches!(result, Err(ServerError::PacHostNotAllowed)));
}

#[tokio::test]
async fn pac_route_returns_not_found_when_disabled() {
    let mut state = multi_config_state(None, 1).await;
    state.pac_enabled = false;

    let result = pac::proxy_pac(State(state), host_headers("127.0.0.1:18203")).await;

    assert!(matches!(result, Err(ServerError::NotFound)));
}

#[tokio::test]
async fn pac_route_accepts_configured_allowed_host() {
    let mut state = multi_config_state(None, 1).await;
    state.pac_allowed_hosts.push("pac.example.test".to_string());

    let response = pac::proxy_pac(State(state), host_headers("pac.example.test:18203"))
        .await
        .expect("configured host should be accepted");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

fn host_headers(host: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::HOST,
        HeaderValue::from_str(host).expect("host header should be valid"),
    );
    headers
}
