use super::super::test_support::{test_context, test_node, test_source};
use super::super::*;
use crate::app::daemon::supervisor::{RuntimeReplaceResult, SupervisorEvent, SupervisorState};
use crate::db::{ConnectionTestInsert, RuntimeSessionInsert};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Child, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

fn write_fake_runtime_script(context: &crate::app::runtime::AppContext) {
    let fake_xray = context.runtime_paths.root_dir.join("fake-xray.py");
    fs::write(
        &fake_xray,
        r#"#!/usr/bin/env python3
import json
import signal
import socket
import sys
import time

config_path = None
for i, arg in enumerate(sys.argv):
    if arg == "-c" and i + 1 < len(sys.argv):
        config_path = sys.argv[i + 1]
        break
if config_path is None:
    sys.exit(2)

with open(config_path, "r", encoding="utf-8") as f:
    cfg = json.load(f)

inbound = cfg["inbounds"][0]
host = inbound.get("listen", "127.0.0.1")
port = int(inbound["port"])
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
sock.bind((host, port))
sock.listen(1)

def _shutdown(*_args):
    sock.close()
    sys.exit(0)

signal.signal(signal.SIGTERM, _shutdown)
signal.signal(signal.SIGINT, _shutdown)

while True:
    time.sleep(1)
"#,
    )
    .expect("fake runtime script should write");

    let mut perms = fs::metadata(&fake_xray)
        .expect("fake runtime script metadata should load")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&fake_xray, perms).expect("fake runtime script should be executable");
}

fn spawn_sleep(seconds: u64) -> Child {
    Command::new("sleep")
        .arg(seconds.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep process should spawn")
}

async fn set_running_session(
    context: &crate::app::runtime::AppContext,
    config_id: i64,
    pid: i64,
) -> i64 {
    context
        .db
        .set_active_config(config_id)
        .await
        .expect("active config should be set");
    context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config_id),
            status: crate::db::RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(pid),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert")
}

#[tokio::test]
async fn health_tick_cooldown_blocks_health_replace_candidate_selection() {
    let context = test_context("health-tick-cooldown-block").await;
    context
        .db
        .import_nodes(
            &test_source(),
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
    assert_eq!(state.last_result, "rotation_no_candidate");
    assert_eq!(state.last_candidate_result, "rotation_no_candidate");
    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::server::RotationTrigger::HealthCheckFailed)
    );
}

#[tokio::test]
async fn health_tick_sets_cooldown_active_when_failure_is_suppressed() {
    let context = test_context("health-tick-cooldown-suppressed").await;
    context
        .db
        .import_nodes(&test_source(), &[test_node("example-a.com", "a")])
        .await
        .expect("node should import");
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");
    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should set");
    let session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
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
        .expect("session should insert");
    context
        .db
        .update_runtime_session_failure_tracking(
            session_id,
            Some(&(u64::MAX - 1).to_string()),
            Some("1"),
            Some("health_check_failed"),
        )
        .await
        .expect("failure tracking should update");

    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.health_trigger_enabled = true;
    handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

    assert!(state.cooldown_active);
    assert!(state.last_trigger.is_none());
}

#[tokio::test]
async fn health_tick_timer_due_attempt_updates_rotation_state_on_failure() {
    let context = test_context("health-tick-timer-due").await;
    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.next_timer_epoch_secs = Some(1);

    handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::server::RotationTrigger::Timer)
    );
    assert_eq!(state.last_result, "rotation_candidate_failed");
    assert_eq!(state.last_candidate_result, "rotation_candidate_failed");
}

#[tokio::test]
async fn manual_replace_failure_persists_rotation_reason_code_on_active_session() {
    let context = test_context("manual-replace-reason-code").await;
    context
        .db
        .import_nodes(&test_source(), &[test_node("example-a.com", "a")])
        .await
        .expect("node should import");
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");
    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should set");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be valid")
        .as_secs()
        .to_string();
    let session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: crate::db::RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(i64::from(std::process::id())),
            failure_reason: None,
            started_at: Some(now),
            stopped_at: None,
        })
        .await
        .expect("active session should insert");

    let mut state = SupervisorState::new("daemon-test".to_string());
    let (tx, rx) = oneshot::channel();
    handle_event(
        &mut state,
        SupervisorEvent::RuntimeReplace {
            trigger: crate::app::daemon::server::RotationTrigger::Manual,
            candidate_id: Some(-1),
            respond_to: tx,
        },
        &context,
    )
    .await;

    match rx.await.expect("replace response should arrive") {
        RuntimeReplaceResult::Err { message } => {
            assert!(message.contains("config -1 was not found"));
        }
        other => panic!("expected replace error, got {other:?}"),
    }

    let session = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("session should load")
        .expect("session should exist");
    assert_eq!(session.id, session_id);
    assert_eq!(
        session.last_transition_reason_code.as_deref(),
        Some("rotation_candidate_failed")
    );
    assert_eq!(state.last_result, "rotation_candidate_failed");
    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::server::RotationTrigger::Manual)
    );
}

#[tokio::test]
async fn proxy_status_reports_candidate_and_cooldown_fields() {
    let context = test_context("proxy-status-fields").await;
    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.last_trigger = Some(crate::app::daemon::server::RotationTrigger::Timer);
    state.last_result = "rotation_no_candidate".to_string();
    state.last_candidate_config_id = Some(42);
    state.last_candidate_result = "rotation_no_candidate".to_string();
    state.cooldown_active = true;

    let (tx, rx) = oneshot::channel();
    handle_event(
        &mut state,
        SupervisorEvent::ProxyStatus { respond_to: tx },
        &context,
    )
    .await;
    let payload = match rx.await.expect("proxy status should arrive") {
        crate::app::daemon::supervisor::ProxyStatusResult::Ok(payload) => payload,
        other => panic!("expected proxy status payload, got {other:?}"),
    };
    assert_eq!(payload.last_candidate_config_id, Some(42));
    assert_eq!(payload.last_candidate_result, "rotation_no_candidate");
    assert!(payload.cooldown_active);
}

#[tokio::test]
async fn health_tick_timer_due_success_updates_rotation_state_and_reschedules() {
    let mut context = test_context("health-tick-timer-success").await;
    context
        .db
        .import_nodes(
            &test_source(),
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
    let candidate_config = configs[1].clone();

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");
    context.app_config.runtime.rotation.test_stages = Vec::new();

    context
        .db
        .insert_connection_test(&ConnectionTestInsert {
            run_id: None,
            config_id: candidate_config.id,
            icmp_ok: None,
            icmp_ms: None,
            tcp_ok: Some(true),
            tcp_ms: Some(5),
            real_delay_ok: Some(true),
            real_delay_ms: Some(20),
            download_mbps: Some(120.0),
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
        .expect("candidate test result should insert");

    let mut old = spawn_sleep(30);
    let old_pid = i64::from(old.id());
    let _old_session_id = set_running_session(&context, active_config.id, old_pid).await;

    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.health_trigger_enabled = false;
    state.rotation_interval_secs = 600;
    state.next_timer_epoch_secs = Some(1);

    handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::server::RotationTrigger::Timer)
    );
    assert_eq!(state.last_result, "replace_commit_success");
    assert_eq!(state.last_candidate_result, "replace_commit_success");
    assert!(!state.cooldown_active);
    assert!(state.next_timer_epoch_secs.is_some());

    let running = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running session should load")
        .expect("running session should exist");
    assert_eq!(
        state.last_candidate_config_id, running.config_id,
        "candidate tracking should match the active session after handoff"
    );
    assert_eq!(
        running.last_transition_reason_code.as_deref(),
        Some("replace_commit_success")
    );
    assert_ne!(running.process_id, Some(old_pid));

    let _ = crate::xray::runtime::terminate_process_gracefully(
        running.process_id.unwrap_or_default(),
        std::time::Duration::from_millis(1500),
    );
    let _ = old.kill();
    let _ = old.wait();
}

#[tokio::test]
async fn manual_rotate_with_explicit_candidate_overrides_cooldown() {
    let mut context = test_context("manual-rotate-cooldown-override").await;
    context
        .db
        .import_nodes(
            &test_source(),
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
    let candidate_config = configs[1].clone();

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");

    let mut old = spawn_sleep(30);
    let old_pid = i64::from(old.id());
    let _old_session_id = set_running_session(&context, active_config.id, old_pid).await;

    let candidate_failed_session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(candidate_config.id),
            status: crate::db::RuntimeSessionStatus::Failed,
            socks_host: None,
            socks_port: None,
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: None,
            failure_reason: Some("health check failed".to_string()),
            started_at: None,
            stopped_at: Some("1".to_string()),
        })
        .await
        .expect("candidate failed session should insert");
    context
        .db
        .update_runtime_session_failure_tracking(
            candidate_failed_session_id,
            Some(&(u64::MAX - 1).to_string()),
            Some("1"),
            Some("health_check_failed"),
        )
        .await
        .expect("candidate cooldown should update");

    let mut state = SupervisorState::new("daemon-test".to_string());
    let (tx, rx) = oneshot::channel();
    handle_event(
        &mut state,
        SupervisorEvent::RuntimeReplace {
            trigger: crate::app::daemon::server::RotationTrigger::Manual,
            candidate_id: Some(candidate_config.id),
            respond_to: tx,
        },
        &context,
    )
    .await;
    let payload = match rx.await.expect("replace response should arrive") {
        RuntimeReplaceResult::Ok(payload) => payload,
        other => panic!("expected replace success, got {other:?}"),
    };

    assert_eq!(payload.new_config_id, candidate_config.id);
    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::server::RotationTrigger::Manual)
    );
    assert_eq!(state.last_result, "replace_commit_success");
    assert_eq!(state.last_candidate_config_id, Some(candidate_config.id));
    assert_eq!(state.last_candidate_result, "replace_commit_success");

    let _ = crate::xray::runtime::terminate_process_gracefully(
        i64::from(payload.new_pid),
        std::time::Duration::from_millis(1500),
    );
    let _ = old.kill();
    let _ = old.wait();
}
