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
        is_selected: false,
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
