use axum::Json;
use axum::extract::{Path, Query, State};

use super::{multi_config_state, populated_state};
use crate::db::{Database, DatabaseConnectionConfig, ImportSource, SourceKind};
use crate::server::ServerError;
use crate::server::ServerState;
use crate::server::routes::configs::{self, ConfigsQuery};

#[tokio::test]
async fn configs_routes_return_list_and_detail() {
    let state = populated_state(None).await;

    let Json(list) = configs::list_configs(
        State(state.clone()),
        Query(ConfigsQuery {
            key: None,
            page: None,
            per_page: None,
            enabled: None,
            protocol: None,
        }),
    )
    .await
    .expect("list route should succeed");
    assert_eq!(list.total, 1);
    assert!(!list.items[0].r#ref.is_empty());
    assert_eq!(
        list.items[0].latest_test.as_ref().unwrap().real_delay_ms,
        Some(123)
    );

    let Json(detail) = configs::get_config(
        State(state),
        Path(list.items[0].id.to_string()),
        Query(ConfigsQuery {
            key: None,
            page: None,
            per_page: None,
            enabled: None,
            protocol: None,
        }),
    )
    .await
    .expect("detail route should succeed");
    assert_eq!(detail.protocol, "vless");
    assert_eq!(detail.r#ref, list.items[0].r#ref);
    assert_eq!(detail.latest_test.unwrap().tcp_ms, Some(45));
}

#[tokio::test]
async fn config_detail_accepts_ref_prefix() {
    let state = populated_state(None).await;
    let config = state
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");
    let prefix = config.r#ref[..8].to_string();

    let Json(detail) = configs::get_config(
        State(state),
        Path(prefix),
        Query(ConfigsQuery {
            key: None,
            page: None,
            per_page: None,
            enabled: None,
            protocol: None,
        }),
    )
    .await
    .expect("detail route should resolve ref prefix");

    assert_eq!(detail.id, config.id);
    assert_eq!(detail.r#ref, config.r#ref);
}

#[tokio::test]
async fn config_detail_returns_not_found_for_missing_id() {
    let state = populated_state(None).await;

    let result = configs::get_config(
        State(state),
        Path("-1".to_string()),
        Query(ConfigsQuery {
            key: None,
            page: None,
            per_page: None,
            enabled: None,
            protocol: None,
        }),
    )
    .await;

    assert!(matches!(result, Err(ServerError::NotFound)));
}

#[tokio::test]
async fn configs_route_pagination_bounds() {
    let state = multi_config_state(None, 5).await;

    let Json(page) = configs::list_configs(
        State(state.clone()),
        Query(ConfigsQuery {
            key: None,
            page: Some(1),
            per_page: Some(2),
            enabled: None,
            protocol: None,
        }),
    )
    .await
    .expect("page 1 should succeed");
    assert_eq!(page.total, 5);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.page, 1);
    assert_eq!(page.per_page, 2);

    let Json(page3) = configs::list_configs(
        State(state.clone()),
        Query(ConfigsQuery {
            key: None,
            page: Some(3),
            per_page: Some(2),
            enabled: None,
            protocol: None,
        }),
    )
    .await
    .expect("page 3 should succeed");
    assert_eq!(page3.total, 5);
    assert_eq!(page3.items.len(), 1);
}

#[tokio::test]
async fn configs_route_rejects_invalid_per_page() {
    let state = populated_state(None).await;

    let result = configs::list_configs(
        State(state.clone()),
        Query(ConfigsQuery {
            key: None,
            page: None,
            per_page: Some(0),
            enabled: None,
            protocol: None,
        }),
    )
    .await;
    assert!(matches!(result, Err(ServerError::InvalidQuery(_))));

    let result = configs::list_configs(
        State(state),
        Query(ConfigsQuery {
            key: None,
            page: None,
            per_page: Some(201),
            enabled: None,
            protocol: None,
        }),
    )
    .await;
    assert!(matches!(result, Err(ServerError::InvalidQuery(_))));
}

#[tokio::test]
async fn config_detail_returns_null_latest_test_when_no_test_exists() {
    let db = Database::connect(&DatabaseConnectionConfig::Sqlite {
        path: std::env::temp_dir().join(format!(
            "xrat-server-no-test-{}.sqlite",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        )),
    })
    .await
    .expect("database should connect");

    db.import_nodes(
        &ImportSource {
            kind: SourceKind::RawText,
            value: "test".to_string(),
            name: None,
        },
        &[super::test_node()],
    )
    .await
    .expect("node should import");

    let config = db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");

    let state = ServerState { db, api_key: None };

    let Json(detail) = configs::get_config(
        State(state),
        Path(config.id.to_string()),
        Query(ConfigsQuery {
            key: None,
            page: None,
            per_page: None,
            enabled: None,
            protocol: None,
        }),
    )
    .await
    .expect("detail should succeed");

    assert!(detail.latest_test.is_none());
    assert_eq!(detail.r#ref, config.r#ref);
}
