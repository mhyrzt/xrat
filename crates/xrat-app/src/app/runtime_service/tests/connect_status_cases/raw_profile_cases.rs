use super::*;

const PROFILE: &[u8] = include_bytes!("../../../../../../../testdata/xray/full-profile.json");

struct NativeRuntimeGuard(Option<u32>);

impl Drop for NativeRuntimeGuard {
    fn drop(&mut self) {
        if let Some(pid) = self.0 {
            let _ = xray_runtime::terminate_process_gracefully(i64::from(pid), SHUTDOWN_TIMEOUT);
        }
    }
}

#[tokio::test]
#[ignore = "requires XRAT_RUNTIME_XRAY and local sockets"]
async fn native_raw_xray_profile_preserves_json_and_proxies_local_traffic() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut context = test_context().await;
    context.app_config.runtime.replace_active_session = true;
    context.runtime_paths.xray_path = std::env::var_os("XRAT_RUNTIME_XRAY")
        .expect("XRAT_RUNTIME_XRAY is required")
        .into();
    let target = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_port = target.local_addr().unwrap().port();
    let reserve = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let proxy_port = reserve.local_addr().unwrap().port();
    drop(reserve);
    let bytes = String::from_utf8(PROFILE.to_vec())
        .unwrap()
        .replace("10899", &proxy_port.to_string())
        .into_bytes();
    let config = raw_config(&context, &bytes).await;
    let service = RuntimeService::new(&context);
    let connected = service
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await
        .unwrap();
    let mut guard = NativeRuntimeGuard(Some(connected.pid));
    let invalid_bytes = String::from_utf8(bytes.clone())
        .unwrap()
        .replace("\"vless\"", "\"unknown-protocol-fixture\"");
    let invalid = raw_config(&context, invalid_bytes.as_bytes()).await;
    let error = service
        .connect(ConnectRequest {
            config_id: invalid.id,
        })
        .await
        .unwrap_err();
    assert!(error.to_string().contains("native validation"));
    assert_eq!(context.db.get_runtime_session_count().await.unwrap(), 1);
    assert_eq!(
        context
            .db
            .get_running_runtime_session()
            .await
            .unwrap()
            .unwrap()
            .process_id,
        Some(i64::from(connected.pid))
    );
    let traffic = tokio::time::timeout(Duration::from_secs(10), async {
        let target_task = tokio::spawn(async move {
            let (mut stream, _) = target.accept().await.unwrap();
            let mut payload = [0; 17];
            stream.read_exact(&mut payload).await.unwrap();
            assert_eq!(&payload, b"profile-roundtrip");
            stream.write_all(b"ok").await.unwrap();
        });
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", proxy_port))
            .await
            .unwrap();
        stream.write_all(&[5, 1, 0]).await.unwrap();
        let mut hello = [0; 2];
        stream.read_exact(&mut hello).await.unwrap();
        assert_eq!(hello, [5, 0]);
        let mut request = vec![5, 1, 0, 1, 127, 0, 0, 1];
        request.extend_from_slice(&target_port.to_be_bytes());
        stream.write_all(&request).await.unwrap();
        let mut reply = [0; 10];
        stream.read_exact(&mut reply).await.unwrap();
        assert_eq!(reply[1], 0);
        stream.write_all(b"profile-roundtrip").await.unwrap();
        let mut response = [0; 2];
        stream.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"ok");
        target_task.await.unwrap();
    })
    .await;
    let running = service.status().await.unwrap();
    assert!(
        service
            .reconcile_reattach_on_daemon_start("raw-profile-restart")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        context
            .db
            .get_running_runtime_session()
            .await
            .unwrap()
            .unwrap()
            .process_id,
        Some(i64::from(connected.pid))
    );
    let stopped = service.disconnect().await.unwrap();
    guard.0 = None;
    traffic.unwrap();
    assert!(running.pid_running);
    assert!(stopped.stopped_session);
    assert_eq!(std::fs::read(connected.runtime_config_path).unwrap(), bytes);

    use crate::app::daemon::supervisor;
    let (sender, receiver) = tokio::sync::mpsc::channel(8);
    let daemon = tokio::spawn(supervisor::run(receiver, context.clone()));
    let (respond_to, response) = tokio::sync::oneshot::channel();
    sender
        .send(supervisor::SupervisorEvent::RuntimeConnect {
            config_id: config.id,
            respond_to,
        })
        .await
        .unwrap();
    let supervisor::RuntimeConnectResult::Ok(connected) = response.await.unwrap() else {
        panic!("daemon connect failed")
    };
    guard.0 = Some(connected.pid);
    let (respond_to, response) = tokio::sync::oneshot::channel();
    sender
        .send(supervisor::SupervisorEvent::ProxyStatus { respond_to })
        .await
        .unwrap();
    let supervisor::ProxyStatusResult::Ok(status) = response.await.unwrap() else {
        panic!("daemon status failed")
    };
    assert!(!status.rotation_enabled);
    assert_eq!(status.last_result, "rotation_paused_for_xray_profile");
    let (respond_to, response) = tokio::sync::oneshot::channel();
    sender
        .send(supervisor::SupervisorEvent::ProxyStart { respond_to })
        .await
        .unwrap();
    assert!(matches!(
        response.await.unwrap(),
        supervisor::ProxyControlResult::Err { .. }
    ));
    let session = context
        .db
        .get_running_runtime_session()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.owner_kind.as_deref(), Some("daemon"));
    assert_eq!(
        std::fs::read(
            context
                .runtime_paths
                .runtime_dir
                .join(format!("session-{}.json", connected.session_id))
        )
        .unwrap(),
        bytes
    );
    let (respond_to, response) = tokio::sync::oneshot::channel();
    sender
        .send(supervisor::SupervisorEvent::DaemonShutdown { respond_to })
        .await
        .unwrap();
    assert!(matches!(
        response.await.unwrap(),
        supervisor::DaemonShutdownResult::Ok(_)
    ));
    daemon.await.unwrap();
    guard.0 = None;
    assert_eq!(
        context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap()
            .status,
        RuntimeSessionStatus::Stopped
    );
}

async fn raw_config(context: &AppContext, bytes: &[u8]) -> ConfigRecord {
    let summary = crate::app::services::catalog_import::persist_content(
        context,
        xrat_db::ImportSource {
            kind: xrat_db::SourceKind::RawText,
            value: "fixture".into(),
            name: None,
        },
        bytes,
        Some("london-vpn"),
        &Default::default(),
    )
    .await
    .unwrap();
    context
        .db
        .list_configs(&ConfigListFilter {
            subscription_id: Some(summary.subscription_id),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_iter()
        .find(ConfigRecord::is_xray_profile)
        .unwrap()
}

#[tokio::test]
async fn raw_profile_preflight_launch_status_and_stop_preserve_original_bytes() {
    let context = test_context().await;
    let config = raw_config(&context, PROFILE).await;
    let (ports, state) = ports(false, false);
    let service = RuntimeService::with_process_ports(&context, ports);
    let connected = service
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(&connected.runtime_config_path).unwrap(),
        PROFILE
    );
    assert_eq!(
        state.lock().unwrap().validated_configs,
        vec![PROFILE.to_vec()]
    );
    let status = service.status().await.unwrap();
    assert_eq!(status.session_config.unwrap().id, config.id);
    assert!(status.pid_running);
    assert_eq!(status.inbound_health.socks.unwrap().endpoint.port, 10899);
    let reference = xrat_model::ConfigRef::from(config.r#ref.clone());
    assert!(
        context
            .services()
            .lifecycle
            .delete(&reference, true)
            .await
            .unwrap_err()
            .to_string()
            .contains("disconnect")
    );
    assert!(service.disconnect().await.unwrap().stopped_session);
    assert_eq!(state.lock().unwrap().spawned, 1);
}

#[tokio::test]
async fn invalid_raw_profile_preflight_preserves_healthy_connection() {
    let mut context = test_context().await;
    let config = import_single_config(&context).await;
    context.app_config.runtime.replace_active_session = true;
    let (mut ports, state) = ports(false, false);
    let previous = RuntimeService::with_process_ports(&context, ports.clone())
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await
        .unwrap();
    let raw = raw_config(&context, PROFILE).await;
    ports.spawner = Arc::new(FakePorts {
        state: state.clone(),
        fail_readiness: false,
        fail_validation: true,
    });
    assert!(
        RuntimeService::with_process_ports(&context, ports.clone())
            .connect(ConnectRequest { config_id: raw.id })
            .await
            .unwrap_err()
            .to_string()
            .contains("validation")
    );
    assert_eq!(
        context
            .db
            .get_running_runtime_session()
            .await
            .unwrap()
            .unwrap()
            .id,
        previous.session_id
    );
    assert_eq!(state.lock().unwrap().spawned, 1);
    assert!(
        !state
            .lock()
            .unwrap()
            .signals
            .iter()
            .any(|signal| matches!(signal, ProcessSignal::Term | ProcessSignal::Kill))
    );
    RuntimeService::with_process_ports(&context, ports)
        .disconnect()
        .await
        .unwrap();
}

#[tokio::test]
async fn raw_profiles_reject_incompatible_modes_and_missing_readiness_without_spawning() {
    let context = test_context().await;
    let config = raw_config(&context, PROFILE).await;
    let (ports, state) = ports(false, false);
    for mode in ["engine", "tun", "killswitch"] {
        let mut candidate = context.clone();
        match mode {
            "engine" => candidate.app_config.runtime.engine = "sing-box".into(),
            "tun" => candidate.app_config.runtime.tun.enabled = true,
            _ => {
                candidate.app_config.runtime.tun.kill_switch =
                    crate::app::config::TunKillSwitchMode::Strict
            }
        }
        assert!(
            RuntimeService::with_process_ports(&candidate, ports.clone())
                .connect(ConnectRequest {
                    config_id: config.id
                })
                .await
                .is_err()
        );
    }
    context
        .db
        .set_config_enabled(config.id, false)
        .await
        .unwrap();
    assert!(
        RuntimeService::with_process_ports(&context, ports.clone())
            .connect(ConnectRequest {
                config_id: config.id
            })
            .await
            .is_err()
    );
    context.db.delete_config(config.id).await.unwrap();
    let empty = raw_config(&context, br#"{"outbounds":[]}"#).await;
    assert!(
        RuntimeService::with_process_ports(&context, ports)
            .connect(ConnectRequest {
                config_id: empty.id
            })
            .await
            .unwrap_err()
            .to_string()
            .contains("readiness")
    );
    assert_eq!(state.lock().unwrap().spawned, 0);
}
