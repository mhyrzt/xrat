use axum::Json;

use crate::server::routes::health;

#[tokio::test]
async fn health_route_returns_ok_without_state_or_auth() {
    let Json(response) = health::health().await;

    assert_eq!(response.status, "ok");
}
