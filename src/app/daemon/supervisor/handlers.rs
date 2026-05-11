use crate::app::daemon::server::{
    DaemonShutdownPayload, PingPayload, RuntimeConnectPayload, RuntimeDisconnectPayload,
    RuntimeReplacePayload, RuntimeStatusPayload,
};
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, RuntimeConnectResult, RuntimeDisconnectResult, RuntimeReplaceResult,
    RuntimeStatusResult, SupervisorEvent, SupervisorState,
};
use crate::app::runtime::AppContext;
use crate::app::runtime_service::{ConnectRequest, ReplaceRequest, RuntimeService};
use std::time::{SystemTime, UNIX_EPOCH};

const HEALTH_FAILURE_COOLDOWN_SECONDS: u64 = 300;

pub async fn handle_event(
    state: &mut SupervisorState,
    event: SupervisorEvent,
    context: &AppContext,
) {
    match event {
        SupervisorEvent::HealthTick => {
            if let Ok(snapshot) = RuntimeService::new(context).status().await {
                if let Some(session) = snapshot.session {
                    if snapshot.pid_running && snapshot.inbound_health.has_unreachable_endpoint() {
                        if !should_record_health_failure(&session) {
                            return;
                        }
                        let failed_at = now_epoch_seconds();
                        let cooldown_until =
                            (failed_at + HEALTH_FAILURE_COOLDOWN_SECONDS).to_string();
                        let failed_at = failed_at.to_string();
                        let _ = context
                            .db
                            .update_runtime_session_transition_metadata(
                                session.id,
                                Some("daemon"),
                                Some(&state.instance_id),
                                Some("health_check_failed"),
                                Some("runtime health check detected unreachable inbound endpoint"),
                                Some("daemon"),
                            )
                            .await;
                        let _ = context
                            .db
                            .update_runtime_session_failure_tracking(
                                session.id,
                                Some(&cooldown_until),
                                Some(&failed_at),
                                Some("health_check_failed"),
                            )
                            .await;
                    }
                }
            }
        }
        SupervisorEvent::DaemonPing { respond_to } => {
            state.ready = true;
            let _ = respond_to.send(PingPayload {
                daemon_ready: state.ready,
            });
        }
        SupervisorEvent::RuntimeStatus { respond_to } => {
            match RuntimeService::new(context).status().await {
                Ok(snapshot) => {
                    let _ = respond_to.send(RuntimeStatusResult::Ok(RuntimeStatusPayload {
                        daemon_ready: state.ready,
                        runtime_owned: snapshot.session.is_some() && snapshot.pid_running,
                        runtime_status: snapshot.status.as_str().to_string(),
                        session_id: snapshot.session.as_ref().map(|session| session.id),
                        active_config_id: snapshot.active_config.as_ref().map(|config| config.id),
                        pid_running: snapshot.pid_running,
                    }));
                }
                Err(err) => {
                    let _ = respond_to.send(RuntimeStatusResult::Err {
                        message: err.to_string(),
                    });
                }
            }
        }
        SupervisorEvent::RuntimeConnect {
            config_id,
            respond_to,
        } => {
            match RuntimeService::new(context)
                .connect(ConnectRequest { config_id })
                .await
            {
                Ok(result) => {
                    let _ = context
                        .db
                        .update_runtime_session_transition_metadata(
                            result.session_id,
                            Some("daemon"),
                            Some(&state.instance_id),
                            Some("manual_connect"),
                            Some("daemon runtime connect request succeeded"),
                            Some("daemon"),
                        )
                        .await;
                    let _ = respond_to.send(RuntimeConnectResult::Ok(RuntimeConnectPayload {
                        config_id: result.config.id,
                        session_id: result.session_id,
                        pid: result.pid,
                    }));
                }
                Err(err) => {
                    let _ = respond_to.send(RuntimeConnectResult::Err {
                        message: err.to_string(),
                    });
                }
            }
        }
        SupervisorEvent::RuntimeDisconnect { respond_to } => {
            let active_session_id = context
                .db
                .get_running_runtime_session()
                .await
                .ok()
                .flatten()
                .map(|session| session.id);
            match RuntimeService::new(context).disconnect().await {
                Ok(result) => {
                    if result.stopped_session {
                        if let Some(session_id) = active_session_id {
                            let _ = context
                                .db
                                .update_runtime_session_transition_metadata(
                                    session_id,
                                    Some("daemon"),
                                    Some(&state.instance_id),
                                    Some("manual_disconnect"),
                                    Some("daemon runtime disconnect request succeeded"),
                                    Some("daemon"),
                                )
                                .await;
                        }
                    }
                    let _ =
                        respond_to.send(RuntimeDisconnectResult::Ok(RuntimeDisconnectPayload {
                            stopped_session: result.stopped_session,
                        }));
                }
                Err(err) => {
                    let _ = respond_to.send(RuntimeDisconnectResult::Err {
                        message: err.to_string(),
                    });
                }
            }
        }
        SupervisorEvent::RuntimeReplace {
            trigger,
            candidate_id,
            respond_to,
        } => {
            match RuntimeService::new(context)
                .replace(ReplaceRequest {
                    trigger,
                    candidate_id,
                })
                .await
            {
                Ok(result) => {
                    let _ = context
                        .db
                        .update_runtime_session_transition_metadata(
                            result.new_session_id,
                            Some("daemon"),
                            Some(&state.instance_id),
                            Some("replace_commit_success"),
                            Some("daemon replace handoff completed"),
                            Some("daemon"),
                        )
                        .await;
                    let _ = respond_to.send(RuntimeReplaceResult::Ok(RuntimeReplacePayload {
                        trigger,
                        replaced: true,
                        old_session_id: result.old_session_id,
                        new_config_id: result.new_config_id,
                        new_session_id: result.new_session_id,
                        new_pid: result.new_pid,
                    }));
                }
                Err(err) => {
                    let _ = respond_to.send(RuntimeReplaceResult::Err {
                        message: err.to_string(),
                    });
                }
            }
        }
        SupervisorEvent::DaemonShutdown { respond_to } => {
            let runtime_disconnected = RuntimeService::new(context)
                .disconnect()
                .await
                .map(|result| result.stopped_session)
                .unwrap_or(false);
            let _ = respond_to.send(DaemonShutdownResult::Ok(DaemonShutdownPayload {
                daemon_ready: false,
                runtime_disconnected,
            }));
        }
    }
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn should_record_health_failure(session: &crate::db::RuntimeSessionRecord) -> bool {
    if session.last_failed_reason_code.as_deref() != Some("health_check_failed") {
        return true;
    }
    let Some(cooldown_until) = session.cooldown_until.as_deref() else {
        return true;
    };
    let Ok(cooldown_until) = cooldown_until.parse::<u64>() else {
        return true;
    };
    now_epoch_seconds() >= cooldown_until
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::AppConfig;
    use crate::app::runtime::RuntimePaths;
    use crate::db::{
        Database, DatabaseConnectionConfig, ImportSource, RuntimeSessionInsert, SourceKind,
    };
    use crate::model::{Node, Protocol};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::oneshot;

    fn session_with_cooldown(cooldown_until: Option<&str>) -> crate::db::RuntimeSessionRecord {
        crate::db::RuntimeSessionRecord {
            id: 1,
            config_id: Some(1),
            status: crate::db::RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(1),
            failure_reason: None,
            owner_kind: Some("daemon".to_string()),
            owner_instance_id: Some("d1".to_string()),
            last_transition_reason_code: Some("health_check_failed".to_string()),
            last_transition_reason_detail: None,
            last_transition_origin: Some("daemon".to_string()),
            cooldown_until: cooldown_until.map(ToString::to_string),
            last_failed_at: Some("1".to_string()),
            last_failed_reason_code: Some("health_check_failed".to_string()),
            started_at: Some("1".to_string()),
            stopped_at: None,
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        }
    }

    #[test]
    fn suppresses_repeated_health_failure_while_cooldown_active() {
        let session = session_with_cooldown(Some(&(u64::MAX - 1).to_string()));
        assert!(!should_record_health_failure(&session));
    }

    #[test]
    fn allows_health_failure_after_cooldown_expires() {
        let session = session_with_cooldown(Some("1"));
        assert!(should_record_health_failure(&session));
    }

    #[tokio::test]
    async fn health_tick_cooldown_blocks_health_replace_candidate_selection() {
        let context = test_context("health-tick-cooldown-block").await;
        let source = ImportSource {
            kind: SourceKind::RawText,
            value: "test".to_string(),
            name: Some("test".to_string()),
        };
        context
            .db
            .import_nodes(
                &source,
                &[
                    test_node("example-a.com", "a"),
                    test_node("example-b.com", "b"),
                ],
            )
            .await
            .expect("nodes should import");
        let mut configs = context
            .db
            .list_configs(&Default::default())
            .await
            .expect("configs should load");
        configs.sort_by_key(|cfg| cfg.id);
        let active_config = configs[0].clone();
        let cooldown_candidate = configs[1].clone();

        context
            .db
            .set_active_config(cooldown_candidate.id)
            .await
            .expect("cooldown candidate should be active first");
        let cooldown_session_id = context
            .db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(cooldown_candidate.id),
                status: crate::db::RuntimeSessionStatus::Running,
                socks_host: Some("127.0.0.1".to_string()),
                socks_port: Some(9),
                http_host: None,
                http_port: None,
                shadowsocks_host: None,
                shadowsocks_port: None,
                process_id: Some(i64::from(std::process::id())),
                failure_reason: None,
                started_at: Some("1".to_string()),
                stopped_at: None,
            })
            .await
            .expect("cooldown session should insert");

        let mut state = SupervisorState::new("daemon-test".to_string());
        handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

        let cooled = context
            .db
            .get_latest_runtime_session_for_config(cooldown_candidate.id)
            .await
            .expect("cooldown session should load")
            .expect("cooldown session should exist");
        assert_eq!(
            cooled.last_failed_reason_code.as_deref(),
            Some("health_check_failed")
        );
        assert!(cooled.cooldown_until.is_some());

        context
            .db
            .update_runtime_session_state(
                cooldown_session_id,
                crate::db::RuntimeSessionStatus::Failed,
                None,
                None,
                Some("2"),
                Some("simulate handoff away from cooled candidate"),
            )
            .await
            .expect("cooldown session should mark failed");

        context
            .db
            .set_active_config(active_config.id)
            .await
            .expect("active config should switch");
        context
            .db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(active_config.id),
                status: crate::db::RuntimeSessionStatus::Running,
                socks_host: Some("127.0.0.1".to_string()),
                socks_port: Some(1080),
                http_host: None,
                http_port: None,
                shadowsocks_host: None,
                shadowsocks_port: None,
                process_id: Some(i64::from(std::process::id())),
                failure_reason: None,
                started_at: Some("3".to_string()),
                stopped_at: None,
            })
            .await
            .expect("active session should insert");

        let (tx, rx) = oneshot::channel();
        handle_event(
            &mut state,
            SupervisorEvent::RuntimeReplace {
                trigger: crate::app::daemon::server::RotationTrigger::HealthCheckFailed,
                candidate_id: None,
                respond_to: tx,
            },
            &context,
        )
        .await;

        match rx.await.expect("replace response should arrive") {
            RuntimeReplaceResult::Err { message } => {
                assert!(message.contains("no eligible replacement candidate"));
            }
            other => panic!("expected replace error, got {other:?}"),
        }
    }

    async fn test_context(prefix: &str) -> AppContext {
        let root = std::env::temp_dir().join(format!(
            "xrat-supervisor-{prefix}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("root should be created");
        let database_config = DatabaseConnectionConfig::Sqlite {
            path: root.join("db.sqlite"),
        };
        let db = Database::connect(&database_config)
            .await
            .expect("database should connect");
        AppContext {
            db,
            app_config: AppConfig::default(),
            runtime_paths: RuntimePaths {
                root_dir: root.clone(),
                database_config,
                database_path: root.join("db.sqlite"),
                database_label: root.join("db.sqlite").display().to_string(),
                config_path: root.join("config.toml"),
                runtime_dir: root.join("runtime"),
                xray_path: PathBuf::from("xray"),
                v2ray_path: PathBuf::from("v2ray"),
                sing_box_path: PathBuf::from("sing-box"),
            },
        }
    }

    fn test_node(address: &str, name: &str) -> Node {
        Node {
            protocol: Protocol::Vless,
            address: address.to_string(),
            port: 443,
            username: None,
            uuid: Some("00000000-0000-0000-0000-000000000000".to_string()),
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: Some(address.to_string()),
            host: None,
            path: None,
            name: Some(name.to_string()),
            extensions: None,
            raw_config: format!(
                "vless://00000000-0000-0000-0000-000000000000@{address}:443?security=tls#{name}"
            ),
        }
    }
}
