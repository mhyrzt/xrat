mod auth;
mod error;
mod response;
mod routes;
mod state;

use std::net::{IpAddr, SocketAddr};

use axum::Router;
use tokio::net::TcpListener;

use crate::app::config::ServerSettings;
use crate::db::Database;

pub use error::{ServerError, ServerResult};
pub use state::ServerState;

pub fn build_router(state: ServerState) -> Router {
    routes::router().with_state(state)
}

pub async fn serve(db: Database, settings: &ServerSettings) -> crate::app::Result<()> {
    let state = ServerState::from_settings(db, settings)?;
    let addr = parse_bind_addr(&settings.host, settings.port)?;
    let router = build_router(state);
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "HTTP API server listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            if let Err(err) = tokio::signal::ctrl_c().await {
                tracing::warn!(error = %err, "failed to listen for shutdown signal");
            }
            tracing::info!("HTTP API server shutting down");
        })
        .await?;

    Ok(())
}

pub async fn serve_with_shutdown(
    db: Database,
    settings: &ServerSettings,
    shutdown: tokio::sync::oneshot::Receiver<()>,
) -> crate::app::Result<()> {
    let state = ServerState::from_settings(db, settings)?;
    let addr = parse_bind_addr(&settings.host, settings.port)?;
    let router = build_router(state);
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "HTTP API server listening (daemon-managed)");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = shutdown.await;
            tracing::info!("HTTP API server shutting down (daemon-managed)");
        })
        .await?;

    Ok(())
}

pub fn parse_bind_addr_public(host: &str, port: u16) -> crate::app::Result<SocketAddr> {
    parse_bind_addr(host, port)
}

fn parse_bind_addr(host: &str, port: u16) -> crate::app::Result<SocketAddr> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(SocketAddr::new(ip, port));
    }

    format!("{host}:{port}").parse().map_err(|err| {
        crate::app::AppError::InvalidArgument(format!(
            "invalid server bind address {host}:{port}: {err}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::body::to_bytes;
    use axum::extract::{Path, Query, State};
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;

    use super::*;
    use crate::db::{ConnectionTestInsert, DatabaseConnectionConfig, ImportSource, SourceKind};
    use crate::model::{Node, Protocol};
    use crate::server::routes::{b64, configs, health, json};

    #[test]
    fn parses_ipv4_and_ipv6_bind_addresses() {
        assert_eq!(
            parse_bind_addr("127.0.0.1", 8080)
                .expect("ipv4 should parse")
                .to_string(),
            "127.0.0.1:8080"
        );
        assert_eq!(
            parse_bind_addr("::1", 8080)
                .expect("ipv6 should parse")
                .to_string(),
            "[::1]:8080"
        );
    }

    #[tokio::test]
    async fn health_route_returns_ok_without_state_or_auth() {
        let Json(response) = health::health().await;

        assert_eq!(response.status, "ok");
    }

    #[tokio::test]
    async fn json_route_returns_enabled_configs_with_latest_test() {
        let state = populated_state(None).await;

        let Json(response) = json::json(
            State(state),
            Query(json::JsonQuery {
                key: None,
                top: None,
                enabled: None,
                protocol: None,
                selected: None,
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
            Query(json::JsonQuery {
                key: Some("wrong".to_string()),
                top: None,
                enabled: None,
                protocol: None,
                selected: None,
            }),
        )
        .await;

        assert!(matches!(result, Err(ServerError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn b64_route_returns_subscription_text_payload() {
        let state = populated_state(None).await;

        let response = b64::b64(
            State(state),
            Query(json::JsonQuery {
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

    #[tokio::test]
    async fn configs_routes_return_list_and_detail() {
        let state = populated_state(None).await;

        let Json(list) = configs::list_configs(
            State(state.clone()),
            Query(configs::ConfigsQuery {
                key: None,
                page: None,
                per_page: None,
                enabled: None,
                protocol: None,
                selected: None,
            }),
        )
        .await
        .expect("list route should succeed");
        assert_eq!(list.total, 1);
        assert_eq!(
            list.items[0].latest_test.as_ref().unwrap().real_delay_ms,
            Some(123)
        );

        let Json(detail) = configs::get_config(
            State(state),
            Path(list.items[0].id),
            Query(configs::ConfigsQuery {
                key: None,
                page: None,
                per_page: None,
                enabled: None,
                protocol: None,
                selected: None,
            }),
        )
        .await
        .expect("detail route should succeed");
        assert_eq!(detail.protocol, "vless");
        assert_eq!(detail.latest_test.unwrap().tcp_ms, Some(45));
    }

    #[tokio::test]
    async fn config_detail_returns_not_found_for_missing_id() {
        let state = populated_state(None).await;

        let result = configs::get_config(
            State(state),
            Path(-1),
            Query(configs::ConfigsQuery {
                key: None,
                page: None,
                per_page: None,
                enabled: None,
                protocol: None,
                selected: None,
            }),
        )
        .await;

        assert!(matches!(result, Err(ServerError::NotFound)));
    }

    #[tokio::test]
    async fn json_route_top_sorts_by_real_delay_ascending() {
        let state = multi_config_state(None, 3).await;

        let Json(response) = json::json(
            State(state),
            Query(json::JsonQuery {
                key: None,
                top: Some(2),
                enabled: None,
                protocol: None,
                selected: None,
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
            Query(json::JsonQuery {
                key: None,
                top: Some(0),
                enabled: None,
                protocol: None,
                selected: None,
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
            Query(json::JsonQuery {
                key: None,
                top: None,
                enabled: None,
                protocol: Some("vless".to_string()),
                selected: None,
            }),
        )
        .await
        .expect("json protocol filter should succeed");

        assert!(response.iter().all(|item| item.protocol == "vless"));
    }

    #[tokio::test]
    async fn configs_route_pagination_bounds() {
        let state = multi_config_state(None, 5).await;

        let Json(page) = configs::list_configs(
            State(state.clone()),
            Query(configs::ConfigsQuery {
                key: None,
                page: Some(1),
                per_page: Some(2),
                enabled: None,
                protocol: None,
                selected: None,
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
            Query(configs::ConfigsQuery {
                key: None,
                page: Some(3),
                per_page: Some(2),
                enabled: None,
                protocol: None,
                selected: None,
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
            Query(configs::ConfigsQuery {
                key: None,
                page: None,
                per_page: Some(0),
                enabled: None,
                protocol: None,
                selected: None,
            }),
        )
        .await;
        assert!(matches!(result, Err(ServerError::InvalidQuery(_))));

        let result = configs::list_configs(
            State(state),
            Query(configs::ConfigsQuery {
                key: None,
                page: None,
                per_page: Some(201),
                enabled: None,
                protocol: None,
                selected: None,
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
            &[test_node()],
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
            Path(config.id),
            Query(configs::ConfigsQuery {
                key: None,
                page: None,
                per_page: None,
                enabled: None,
                protocol: None,
                selected: None,
            }),
        )
        .await
        .expect("detail should succeed");

        assert!(detail.latest_test.is_none());
    }

    async fn multi_config_state(api_key: Option<&str>, count: usize) -> ServerState {
        let db = Database::connect(&DatabaseConnectionConfig::Sqlite {
            path: std::env::temp_dir().join(format!(
                "xrat-server-multi-{}.sqlite",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("time should be valid")
                    .as_nanos()
            )),
        })
        .await
        .expect("database should connect");

        let nodes: Vec<Node> = (0..count)
            .map(|i| Node {
                protocol: Protocol::Vless,
                address: format!("node-{i}.example.com"),
                port: 443,
                username: None,
                uuid: Some(format!("00000000-0000-0000-0000-{i:012}")),
                password: None,
                method: None,
                network: "tcp".to_string(),
                tls: Some("tls".to_string()),
                sni: None,
                host: None,
                path: None,
                name: Some(format!("node-{i}")),
                extensions: None,
                raw_config: format!(
                    "vless://00000000-0000-0000-0000-{i:012}@node-{i}.example.com:443#node-{i}"
                ),
            })
            .collect();

        db.import_nodes(
            &ImportSource {
                kind: SourceKind::RawText,
                value: "test".to_string(),
                name: None,
            },
            &nodes,
        )
        .await
        .expect("nodes should import");

        let configs = db
            .list_configs(&Default::default())
            .await
            .expect("configs should load");

        for (i, config) in configs.iter().enumerate() {
            db.insert_connection_test(&ConnectionTestInsert {
                run_id: None,
                config_id: config.id,
                icmp_ok: None,
                icmp_ms: None,
                tcp_ok: Some(true),
                tcp_ms: Some(40 + i as i64),
                real_delay_ok: Some(true),
                real_delay_ms: Some((count as i64 - i as i64) * 100),
                download_mbps: None,
                upload_mbps: None,
                connect_ms: None,
                ttfb_ms: None,
                http_status: None,
                endpoint_ip: None,
                endpoint_location: None,
                endpoint_country: None,
                endpoint_asn: None,
                failure_kind: None,
                failure_reason: None,
            })
            .await
            .expect("test should insert");
        }

        ServerState {
            db,
            api_key: api_key.map(str::to_string),
        }
    }

    async fn populated_state(api_key: Option<&str>) -> ServerState {
        let db = Database::connect(&DatabaseConnectionConfig::Sqlite {
            path: std::env::temp_dir().join(format!(
                "xrat-server-route-{}.sqlite",
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
            &[test_node()],
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
        db.insert_connection_test(&ConnectionTestInsert {
            run_id: None,
            config_id: config.id,
            icmp_ok: None,
            icmp_ms: None,
            tcp_ok: Some(true),
            tcp_ms: Some(45),
            real_delay_ok: Some(true),
            real_delay_ms: Some(123),
            download_mbps: None,
            upload_mbps: None,
            connect_ms: None,
            ttfb_ms: None,
            http_status: None,
            endpoint_ip: None,
            endpoint_location: None,
            endpoint_country: None,
            endpoint_asn: None,
            failure_kind: None,
            failure_reason: None,
        })
        .await
        .expect("test should insert");

        ServerState {
            db,
            api_key: api_key.map(str::to_string),
        }
    }

    fn test_node() -> Node {
        Node {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("00000000-0000-0000-0000-000000000001".to_string()),
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("example.com".to_string()),
            host: None,
            path: None,
            name: Some("test-node".to_string()),
            extensions: None,
            raw_config: "vless://00000000-0000-0000-0000-000000000001@example.com:443?security=tls#test-node".to_string(),
        }
    }
}
