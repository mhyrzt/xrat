use axum::Json;
use axum::extract::{Query, State};

use super::{multi_config_state, populated_state};
use crate::server::ServerError;
use crate::server::routes::json::{self, JsonQuery};

#[tokio::test]
async fn json_route_returns_enabled_configs_with_latest_test() {
    let state = populated_state(None).await;

    let Json(response) = json::json(
        State(state),
        Query(JsonQuery {
            key: None,
            top: None,
            enabled: None,
            protocol: None,
        }),
    )
    .await
    .expect("json route should succeed");

    assert_eq!(response.len(), 1);
    assert_eq!(response[0].protocol, "vless");
    assert_eq!(response[0].real_delay_ms, Some(123));
    assert_eq!(response[0].tcp_ok, Some(true));
}

#[tokio::test]
async fn json_route_enforces_api_key() {
    let state = populated_state(Some("secret")).await;

    let result = json::json(
        State(state),
        Query(JsonQuery {
            key: Some("wrong".to_string()),
            top: None,
            enabled: None,
            protocol: None,
        }),
    )
    .await;

    assert!(matches!(result, Err(ServerError::InvalidApiKey)));
}

#[tokio::test]
async fn json_route_top_sorts_by_real_delay_ascending() {
    let state = multi_config_state(None, 3).await;

    let Json(response) = json::json(
        State(state),
        Query(JsonQuery {
            key: None,
            top: Some(2),
            enabled: None,
            protocol: None,
        }),
    )
    .await
    .expect("json top route should succeed");

    assert_eq!(response.len(), 2);
    assert!(response[0].real_delay_ms.unwrap() <= response[1].real_delay_ms.unwrap());
}

#[tokio::test]
async fn json_route_top_zero_returns_error() {
    let state = populated_state(None).await;

    let result = json::json(
        State(state),
        Query(JsonQuery {
            key: None,
            top: Some(0),
            enabled: None,
            protocol: None,
        }),
    )
    .await;

    assert!(matches!(result, Err(ServerError::InvalidQuery(_))));
}

#[tokio::test]
async fn json_route_protocol_filter_restricts_results() {
    let state = multi_config_state(None, 2).await;

    let Json(response) = json::json(
        State(state),
        Query(JsonQuery {
            key: None,
            top: None,
            enabled: None,
            protocol: Some("vless".to_string()),
        }),
    )
    .await
    .expect("json protocol filter should succeed");

    assert!(response.iter().all(|item| item.protocol == "vless"));
}
