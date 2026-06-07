use super::*;

#[test]
fn running_session_with_unreachable_inbound_is_degraded() {
    let session = runtime_session_with_status(RuntimeSessionStatus::Running);
    let health = RuntimeInboundHealth {
        socks: Some(RuntimeEndpointHealth {
            endpoint: RuntimeEndpoint {
                host: "127.0.0.1".to_string(),
                port: 1080,
            },
            state: RuntimeEndpointState::Unreachable,
        }),
        http: None,
        shadowsocks: None,
    };

    assert_eq!(
        runtime_status_label(&Some(session), &ActiveSessionState::None, true, &health),
        RuntimeSessionDisplay::Degraded
    );
}

#[test]
fn running_session_with_reachable_inbounds_keeps_persisted_status() {
    let session = runtime_session_with_status(RuntimeSessionStatus::Running);
    let health = RuntimeInboundHealth {
        socks: Some(RuntimeEndpointHealth {
            endpoint: RuntimeEndpoint {
                host: "127.0.0.1".to_string(),
                port: 1080,
            },
            state: RuntimeEndpointState::Reachable,
        }),
        http: None,
        shadowsocks: None,
    };

    assert_eq!(
        runtime_status_label(&Some(session), &ActiveSessionState::None, true, &health),
        RuntimeSessionDisplay::Persisted(RuntimeSessionStatus::Running)
    );
}

#[test]
fn rejects_unknown_protocol() {
    let record = ConfigRecord {
        id: 1,
        r#ref: "ref000000001".to_string(),
        subscription_id: None,
        dedup_key: "key".to_string(),
        protocol: "unknown".to_string(),
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: None,
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        name: None,
        raw_config: "raw".to_string(),
        is_active: false,
        is_enabled: true,
        is_deleted: false,
        deleted_at: None,
        imported_at: "now".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
    };

    assert!(matches!(
        node_from_record(&record),
        Err(AppError::UnsupportedProtocol(_))
    ));
}

#[test]
fn maps_wildcard_bind_hosts_to_loopback_for_readiness() {
    assert_eq!(connect_host_for_bind_host("0.0.0.0"), "127.0.0.1");
    assert_eq!(connect_host_for_bind_host("::"), "::1");
    assert_eq!(connect_host_for_bind_host("127.0.0.1"), "127.0.0.1");
}

#[tokio::test]
async fn hy2_launch_auto_selects_singbox_runtime() {
    let context = test_context().await;
    let config = imported_config(&context, hy2_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&config)
        .expect("hy2 launch should resolve");

    assert_eq!(launch.binary_path, context.runtime_paths.sing_box_path);
    assert_eq!(launch.ready_port, context.app_config.runtime.socks.port);
    assert!(matches!(launch.config, RuntimeLaunchConfig::Singbox(_)));
    assert_eq!(
        launch.endpoints.socks,
        Some(RuntimeEndpoint {
            host: context.app_config.runtime.socks.host.clone(),
            port: context.app_config.runtime.socks.port,
        })
    );
}

#[tokio::test]
async fn configured_singbox_rejects_non_hy2_until_runtime_generation_exists() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    let config = imported_config(&context, test_node()).await;
    let service = RuntimeService::new(&context);

    let error = match service.resolve_launch(&config) {
        Ok(_) => panic!("non-hy2 sing-box launch should fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("supports hy2 configs only"));
}

fn runtime_session_with_status(status: RuntimeSessionStatus) -> RuntimeSessionRecord {
    RuntimeSessionRecord {
        id: 1,
        config_id: None,
        status,
        socks_host: Some("127.0.0.1".to_string()),
        socks_port: Some(1080),
        http_host: None,
        http_port: None,
        shadowsocks_host: None,
        shadowsocks_port: None,
        process_id: Some(i64::from(std::process::id())),
        failure_reason: None,
        owner_kind: None,
        owner_instance_id: None,
        last_transition_reason_code: None,
        last_transition_reason_detail: None,
        last_transition_origin: None,
        cooldown_until: None,
        last_failed_at: None,
        last_failed_reason_code: None,
        started_at: Some("1".to_string()),
        stopped_at: None,
        created_at: "1".to_string(),
        updated_at: "1".to_string(),
    }
}

async fn imported_config(context: &AppContext, node: crate::model::Node) -> ConfigRecord {
    context
        .db
        .import_nodes(&test_source(), &[node])
        .await
        .expect("node should import");
    context
        .db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("configs should list")
        .into_iter()
        .next()
        .expect("import should create config")
}

fn hy2_node() -> crate::model::Node {
    crate::model::Node {
        protocol: Protocol::Hy2,
        address: "hy2.example.com".to_string(),
        port: 443,
        username: None,
        uuid: None,
        password: Some("secret".to_string()),
        method: None,
        network: "udp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("edge.example.com".to_string()),
        host: None,
        path: None,
        name: Some("hy2".to_string()),
        extensions: None,
        raw_config: "hy2://secret@hy2.example.com:443?sni=edge.example.com#hy2".to_string(),
    }
}
