use super::super::validate_operator;
use super::*;
use crate::app::daemon::supervisor::{RuntimeConnectResult, RuntimeDisconnectResult};
use xrat_model::ConfigId;

const OPERATOR: &str = "S-1-5-21-1-2-3-1001";

async fn fixture() -> (MachineGateway, mpsc::Receiver<SupervisorEvent>) {
    let (supervisor, receiver) = mpsc::channel(4);
    let mut context = crate::app::tests::TestAppBuilder::new("machine-gateway")
        .build()
        .await;
    context.runtime_paths.config_path = "protected/config.toml".into();
    (
        MachineGateway::for_test(context, OPERATOR, supervisor),
        receiver,
    )
}

fn request(request: GatewayRequestKind) -> GatewayRequest {
    GatewayRequest {
        protocol_version: GATEWAY_VERSION,
        request,
    }
}

#[test]
fn operator_sid_rejects_service_accounts_and_invalid_input() {
    assert!(validate_operator(OPERATOR).is_ok());
    for sid in [
        "",
        "S-1-",
        "S-1-5-18",
        "S-1-5-19",
        "S-1-5-20",
        "S-1-5-32-544",
        "S-1-5-21)(A;;GA;;;WD)",
        "S-1-5-4294967296",
    ] {
        assert!(validate_operator(sid).is_err(), "accepted {sid}");
    }
}

#[test]
fn wire_schema_rejects_paths_commands_sql_json_and_extra_fields() {
    for input in [
        r#"{"type":"tun","enabled":true,"config_path":"C:\\untrusted.toml"}"#,
        r#"{"type":"connect","config_id":1,"engine_json":{}}"#,
        r#"{"type":"connect","config_id":1,"executable":"cmd.exe"}"#,
        r#"{"type":"status","sql":"DROP TABLE configs"}"#,
        r#"{"type":"daemon_shutdown"}"#,
        r#"{"type":"replace","trigger":"health_check_failed"}"#,
    ] {
        assert!(
            serde_json::from_str::<GatewayRequestKind>(input).is_err(),
            "accepted {input}"
        );
    }
    assert!(
        serde_json::from_str::<GatewayRequest>(
            r#"{"protocol_version":1,"request":{"type":"status"},"path":"arbitrary"}"#
        )
        .is_err()
    );
}

#[tokio::test]
async fn rejects_other_users_versions_and_ids_before_supervisor_dispatch() {
    let (gateway, mut receiver) = fixture().await;
    for peer in ["S-1-5-21-1-2-3-1002", "S-1-5-18", "S-1-5-32-544", ""] {
        assert!(
            gateway
                .dispatch(peer, request(GatewayRequestKind::Status {}))
                .await
                .is_err()
        );
    }
    let mut wrong_version = request(GatewayRequestKind::Status {});
    wrong_version.protocol_version += 1;
    assert!(gateway.dispatch(OPERATOR, wrong_version).await.is_err());
    for id in [0, -1] {
        assert!(
            gateway
                .dispatch(
                    OPERATOR,
                    request(GatewayRequestKind::Connect {
                        config_id: ConfigId(id)
                    })
                )
                .await
                .is_err()
        );
        assert!(
            gateway
                .dispatch(
                    OPERATOR,
                    request(GatewayRequestKind::Replace {
                        candidate_id: Some(ConfigId(id))
                    })
                )
                .await
                .is_err()
        );
    }
    assert!(receiver.try_recv().is_err());
}

#[tokio::test]
async fn connect_and_repeated_disconnect_preserve_supervisor_results() {
    let (gateway, mut receiver) = fixture().await;
    let worker = tokio::spawn(async move {
        let Some(SupervisorEvent::RuntimeConnect {
            config_id,
            respond_to,
        }) = receiver.recv().await
        else {
            panic!("expected connect")
        };
        assert_eq!(config_id, ConfigId(42));
        respond_to
            .send(RuntimeConnectResult::Err {
                message: "config is disabled".into(),
            })
            .unwrap();
        for stopped_session in [true, false] {
            let Some(SupervisorEvent::RuntimeDisconnect { respond_to }) = receiver.recv().await
            else {
                panic!("expected disconnect")
            };
            respond_to
                .send(RuntimeDisconnectResult::Ok(ipc::RuntimeDisconnectPayload {
                    stopped_session,
                }))
                .unwrap();
        }
    });
    let response = gateway
        .dispatch(
            OPERATOR,
            request(GatewayRequestKind::Connect {
                config_id: ConfigId(42),
            }),
        )
        .await
        .unwrap();
    assert!(!response.ok);
    assert_eq!(response.message, "config is disabled");
    for expected in [true, false] {
        let response = gateway
            .dispatch(OPERATOR, request(GatewayRequestKind::Disconnect {}))
            .await
            .unwrap();
        let Some(GatewayPayload::Disconnect(payload)) = response.payload else {
            panic!("missing disconnect result")
        };
        assert_eq!(payload.stopped_session, expected);
    }
    worker.await.unwrap();
}

#[tokio::test]
async fn tun_uses_only_service_owned_path_and_replace_is_manual() {
    let (gateway, mut receiver) = fixture().await;
    let worker = tokio::spawn(async move {
        let Some(SupervisorEvent::RuntimeTun {
            enabled,
            config_path,
            respond_to,
        }) = receiver.recv().await
        else {
            panic!("expected tun")
        };
        assert_eq!(enabled, Some(true));
        assert_eq!(config_path, PathBuf::from("protected/config.toml"));
        respond_to
            .send(Err("Windows TUN is not supported yet".into()))
            .unwrap();
        let Some(SupervisorEvent::RuntimeReplace {
            trigger,
            candidate_id,
            respond_to,
        }) = receiver.recv().await
        else {
            panic!("expected replace")
        };
        assert_eq!(trigger, ipc::RotationTrigger::Manual);
        assert_eq!(candidate_id, None);
        respond_to
            .send(crate::app::daemon::supervisor::RuntimeReplaceResult::Err {
                message: "no candidates".into(),
            })
            .unwrap();
    });
    let response = gateway
        .dispatch(
            OPERATOR,
            request(GatewayRequestKind::Tun {
                enabled: Some(true),
            }),
        )
        .await
        .unwrap();
    assert!(!response.ok);
    let response = gateway
        .dispatch(
            OPERATOR,
            request(GatewayRequestKind::Replace { candidate_id: None }),
        )
        .await
        .unwrap();
    assert!(!response.ok);
    worker.await.unwrap();
}

#[tokio::test]
async fn closed_supervisor_does_not_fall_back_to_local_execution() {
    let (gateway, receiver) = fixture().await;
    drop(receiver);
    assert!(
        gateway
            .dispatch(OPERATOR, request(GatewayRequestKind::Disconnect {}))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn unauthorized_catalog_mutations_leave_the_shared_catalog_unchanged() {
    use super::super::catalog::{CatalogPayload, CatalogRequest};
    use std::collections::BTreeMap;
    use xrat_db::{ImportSource, SourceKind};
    let (gateway, mut receiver) = fixture().await;
    let mutation = || {
        request(GatewayRequestKind::Catalog {
            request: CatalogRequest::Import {
                contents: vec!["vless://id@one.example:443#first".into()],
                source: ImportSource {
                    kind: SourceKind::RawText,
                    value: "fixture".into(),
                    name: None,
                },
                name: None,
                headers: BTreeMap::new(),
            },
        })
    };
    assert!(
        gateway
            .dispatch("S-1-5-21-1-2-3-1002", mutation())
            .await
            .is_err()
    );
    let mut wrong_version = mutation();
    wrong_version.protocol_version += 1;
    assert!(gateway.dispatch(OPERATOR, wrong_version).await.is_err());
    let response = gateway
        .dispatch(
            OPERATOR,
            request(GatewayRequestKind::Catalog {
                request: CatalogRequest::Subscriptions {
                    offset: 0,
                    limit: 1,
                },
            }),
        )
        .await
        .unwrap();
    let Some(GatewayPayload::Catalog(CatalogPayload::Subscriptions { total, items })) =
        response.payload
    else {
        panic!("expected subscriptions");
    };
    assert_eq!(total, 0);
    assert!(items.is_empty());
    assert!(receiver.try_recv().is_err());
}

#[tokio::test]
async fn diagnostics_authorization_precedes_reads_and_event_clear() {
    use crate::app::services::diagnostics::{DiagnosticsPayload, DiagnosticsRequest};
    let (context, _root) = crate::app::tests::TestAppBuilder::new("gateway-diagnostics")
        .build_with_root()
        .await;
    context
        .db
        .record_event(&xrat_db::NewEvent {
            level: "info".into(),
            source: "runtime".into(),
            kind: "test".into(),
            config_id: None,
            session_id: None,
            message: "kept".into(),
            detail: None,
        })
        .await
        .unwrap();
    let (sender, _receiver) = mpsc::channel(4);
    let gateway = MachineGateway::for_test(context.clone(), OPERATOR, sender);
    for operation in [
        DiagnosticsRequest::Events {
            after: None,
            limit: 1,
            level: None,
        },
        DiagnosticsRequest::ClearEvents {},
    ] {
        assert!(
            gateway
                .dispatch(
                    "S-1-5-21-1-2-3-1002",
                    request(GatewayRequestKind::Diagnostics { request: operation })
                )
                .await
                .is_err()
        );
    }
    assert_eq!(
        context
            .db
            .list_events(&xrat_db::EventFilter {
                limit: 1,
                ..Default::default()
            })
            .await
            .unwrap()
            .len(),
        1
    );
    let result = gateway
        .dispatch(
            OPERATOR,
            request(GatewayRequestKind::Diagnostics {
                request: DiagnosticsRequest::ClearEvents {},
            }),
        )
        .await
        .unwrap();
    assert!(matches!(
        result.payload,
        Some(GatewayPayload::Diagnostics(DiagnosticsPayload::Cleared(1)))
    ));
}

#[tokio::test]
async fn overview_requires_operator_and_uses_only_the_supervisor_result() {
    let (gateway, mut receiver) = fixture().await;
    assert!(
        gateway
            .dispatch(
                "S-1-5-21-1-2-3-1002",
                request(GatewayRequestKind::Overview {})
            )
            .await
            .is_err()
    );
    assert!(receiver.try_recv().is_err());
    let worker = tokio::spawn(async move {
        let Some(SupervisorEvent::RuntimeOverview { respond_to }) = receiver.recv().await else {
            panic!("overview");
        };
        respond_to
            .send(Ok(Box::new(crate::app::read_models::RuntimeOverview {
                runtime: crate::app::runtime_service::RuntimeStatusSnapshot {
                    status: crate::app::runtime_service::RuntimeSessionDisplay::Stopped,
                    session: None,
                    session_config: None,
                    active_config: None,
                    pid_running: false,
                    inbound_health: Default::default(),
                    database_label: "protected catalog".into(),
                },
                tun: ipc::TunStatePayload {
                    enabled: false,
                    active: false,
                    engine: "xray".into(),
                    interface: None,
                    active_config_ref: None,
                    session_id: None,
                    split_mode: "all".into(),
                    blacklist_count: 0,
                    whitelist_count: 0,
                    kill_switch: "off".into(),
                    kill_switch_active: false,
                    allow_lan: true,
                },
                daemon_ready: true,
                rotation_enabled: false,
                rotation_interval_secs: 71,
                test_stages: vec!["tcp".into()],
                config_path: "protected/config.toml".into(),
            })))
            .unwrap();
        let Some(SupervisorEvent::RuntimeOverview { respond_to }) = receiver.recv().await else {
            panic!("overview");
        };
        respond_to
            .send(Err("runtime owner unavailable".into()))
            .unwrap();
    });
    let response = gateway
        .dispatch(OPERATOR, request(GatewayRequestKind::Overview {}))
        .await
        .unwrap();
    let decoded: ipc::DaemonResponse<GatewayPayload> =
        serde_json::from_slice(&serde_json::to_vec(&response).unwrap()).unwrap();
    let Some(GatewayPayload::Overview(overview)) = decoded.payload else {
        panic!("overview payload");
    };
    assert_eq!(overview.runtime.database_label, "protected catalog");
    assert!(overview.daemon_ready);
    assert_eq!(overview.rotation_interval_secs, 71);
    assert_eq!(overview.test_stages, ["tcp"]);
    let error = gateway
        .dispatch(OPERATOR, request(GatewayRequestKind::Overview {}))
        .await
        .unwrap_err();
    assert!(error.to_string().contains("runtime owner unavailable"));
    worker.await.unwrap();
    assert!(
        serde_json::from_str::<GatewayRequestKind>(
            r#"{"type":"overview","config_path":"untrusted"}"#
        )
        .is_err()
    );
}

#[tokio::test]
async fn probe_job_authorization_precedes_scheduling_and_cancellation() {
    use super::super::{ProbeRequest, ProbeStages};
    let (gateway, mut receiver) = fixture().await;
    for operation in [
        ProbeRequest::Start {
            config_ids: vec![ConfigId(1)],
            stages: ProbeStages::default(),
        },
        ProbeRequest::Status { job_id: None },
        ProbeRequest::Cancel {
            job_id: uuid::Uuid::new_v4().to_string(),
        },
    ] {
        let request = request(GatewayRequestKind::Probes { request: operation });
        assert!(
            gateway
                .dispatch("S-1-5-21-1-2-3-1002", request.clone())
                .await
                .is_err()
        );
        let mut wrong_version = request;
        wrong_version.protocol_version += 1;
        assert!(gateway.dispatch(OPERATOR, wrong_version).await.is_err());
    }
    let result_request = request(GatewayRequestKind::ProbeResults {
        job_id: uuid::Uuid::new_v4().to_string(),
        offset: 0,
        limit: 1,
    });
    assert!(
        gateway
            .dispatch("S-1-5-21-1-2-3-1002", result_request.clone())
            .await
            .is_err()
    );
    let mut wrong_version = result_request;
    wrong_version.protocol_version += 1;
    assert!(gateway.dispatch(OPERATOR, wrong_version).await.is_err());
    let response = gateway
        .dispatch(
            OPERATOR,
            request(GatewayRequestKind::Probes {
                request: ProbeRequest::Status { job_id: None },
            }),
        )
        .await
        .unwrap();
    assert!(matches!(
        response.payload,
        Some(GatewayPayload::Probes(None))
    ));
    assert!(receiver.try_recv().is_err());
    gateway.shutdown().await.unwrap();
}

#[path = "tests/telemetry.rs"]
mod telemetry;
