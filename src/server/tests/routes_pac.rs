use axum::body::to_bytes;
use axum::extract::State;
use axum::http::header;

use super::multi_config_state;
use crate::server::routes::pac;

#[tokio::test]
async fn pac_route_sets_content_type_and_is_unauthenticated() {
    // An API key is configured, but the PAC route must not require it.
    let state = multi_config_state(Some("secret"), 1).await;

    let response = pac::proxy_pac(State(state))
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
