use super::super::*;
use super::test_support::{test_context, test_node, test_source};
use crate::app::daemon::server::RotationTrigger;
use crate::xray::runtime as xray_runtime;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

#[tokio::test]
async fn replace_rejects_without_running_session() {
    let context = test_context().await;
    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: None,
        })
        .await;
    match result {
        Err(AppError::InvalidArgument(message)) => {
            assert!(message.contains("no running runtime session to replace"));
        }
        other => panic!("expected invalid argument, got {other:?}"),
    }
}

#[tokio::test]
async fn replace_validation_failure_keeps_old_runtime_active() {
    let context = test_context().await;
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
        .await
        .expect("node should import");
    assert_eq!(summary.imported_configs, 1);
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");

    let mut child = Command::new("sleep")
        .arg("5")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep process should spawn");
    let pid = i64::from(child.id());

    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should be set");
    let session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: RuntimeSessionStatus::Running,
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
        .expect("session should insert");

    // Simulate validation failure by requesting a config that does not exist.
    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: Some(-1),
        })
        .await;

    let _ = child.kill();
    let _ = child.wait();

    match result {
        Err(AppError::InvalidArgument(message)) => {
            assert!(message.contains("config -1 was not found"));
        }
        other => panic!("expected invalid argument, got {other:?}"),
    }

    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_eq!(latest.id, session_id);
    assert_eq!(latest.status, RuntimeSessionStatus::Running);
    assert_eq!(latest.process_id, Some(pid));
    assert_eq!(
        latest.last_transition_reason_code.as_deref(),
        Some("replace_rollback_keep_old")
    );
    assert_eq!(
        latest.last_transition_reason_detail.as_deref(),
        Some("replacement candidate rejected before handoff")
    );
    assert_eq!(latest.last_transition_origin.as_deref(), Some("daemon"));

    let active_config = context
        .db
        .get_active_config()
        .await
        .expect("active config should load")
        .expect("active config should still exist");
    assert_eq!(active_config.id, config.id);
}

#[tokio::test]
async fn replace_spawn_failure_keeps_old_runtime_active() {
    let mut context = test_context().await;
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
        .await
        .expect("node should import");
    assert_eq!(summary.imported_configs, 1);
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");

    let mut child = Command::new("sleep")
        .arg("5")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep process should spawn");
    let pid = i64::from(child.id());

    context.runtime_paths.xray_path = std::path::PathBuf::from("/definitely/missing-xray");
    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should be set");
    let old_session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: RuntimeSessionStatus::Running,
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
        .expect("session should insert");

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: Some(config.id),
        })
        .await;

    let _ = child.kill();
    let _ = child.wait();

    assert!(
        result.is_err(),
        "replace should fail when candidate spawn fails"
    );
    let running = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running should load")
        .expect("old runtime should still be running");
    assert_eq!(running.id, old_session_id);
    assert_eq!(running.status, RuntimeSessionStatus::Running);
    assert_eq!(running.process_id, Some(pid));

    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_ne!(latest.id, old_session_id);
    assert_eq!(latest.status, RuntimeSessionStatus::Failed);
    assert_eq!(
        latest.last_failed_reason_code.as_deref(),
        Some("replace_validation_failed")
    );
    assert!(latest.last_failed_at.is_some());
}

#[tokio::test]
async fn replace_success_stages_new_then_stops_old_runtime() {
    let mut context = test_context().await;
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
        .await
        .expect("node should import");
    assert_eq!(summary.imported_configs, 1);
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");

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

    context.runtime_paths.xray_path = fake_xray;
    let mut old = Command::new("sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("old runtime process should spawn");
    let old_pid = i64::from(old.id());

    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should be set");
    let old_session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(old_pid),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("old session should insert");

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: Some(config.id),
        })
        .await
        .expect("replace should succeed");

    assert_eq!(result.old_session_id, old_session_id);
    assert_ne!(result.new_session_id, old_session_id);
    let mut old_stopped = false;
    for _ in 0..20 {
        if old
            .try_wait()
            .expect("old process wait should succeed")
            .is_some()
        {
            old_stopped = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(old_stopped, "old runtime process should be stopped");

    let running = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running should load")
        .expect("new running session should exist");
    assert_eq!(running.id, result.new_session_id);
    assert_eq!(running.status, RuntimeSessionStatus::Running);
    assert_eq!(running.config_id, Some(config.id));

    assert_eq!(
        running.last_transition_reason_code.as_deref(),
        Some("replace_commit_success")
    );
    assert_eq!(
        running.last_transition_reason_detail.as_deref(),
        Some("runtime replace handoff completed")
    );
    assert_eq!(running.last_transition_origin.as_deref(), Some("daemon"));

    let active_config = context
        .db
        .get_active_config()
        .await
        .expect("active config should load")
        .expect("active config should exist");
    assert_eq!(active_config.id, config.id);

    let _ = xray_runtime::terminate_process_gracefully(result.new_pid as i64, SHUTDOWN_TIMEOUT);
    let _ = old.kill();
    let _ = old.wait();
}
