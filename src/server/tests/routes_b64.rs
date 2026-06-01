use axum::body::to_bytes;
use axum::extract::{Query, State};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::{populated_state, test_node};
use crate::server::routes::{b64, json::JsonQuery};

#[tokio::test]
async fn b64_route_returns_subscription_text_payload() {
    let state = populated_state(None).await;

    let response = b64::b64(
        State(state),
        Query(JsonQuery {
            key: None,
            top: None,
            enabled: None,
            protocol: None,
            selected: None,
        }),
    )
    .await
    .expect("b64 route should succeed");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let decoded = STANDARD.decode(body).expect("body should be valid base64");

    assert_eq!(
        String::from_utf8(decoded).expect("utf8"),
        test_node().raw_config
    );
}
