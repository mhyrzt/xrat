use super::*;
use crate::xray::runtime as xray_runtime;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

#[tokio::test]
async fn connect_rejects_when_runtime_running_and_replace_disabled() {
    let mut context = test_context().await;
    context.app_config.runtime.replace_active_session = false;
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
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
    assert_eq!(summary.imported_configs, 1);

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
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await;
    let _ = child.kill();
    let _ = child.wait();

    assert!(matches!(result, Err(AppError::RuntimeSessionAlreadyActive)));
}

#[tokio::test]
async fn connect_and_disconnect_persist_direct_transition_metadata() {
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

    let connected = RuntimeService::new(&context)
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await
        .expect("connect should succeed");
    assert!(connected.pid > 0);

    let connected_session = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running session should load")
        .expect("running session should exist");
    assert_eq!(
        connected_session.last_transition_reason_code.as_deref(),
        Some("manual_connect")
    );
    assert_eq!(
        connected_session.last_transition_reason_detail.as_deref(),
        Some("runtime connect request succeeded")
    );
    assert_eq!(
        connected_session.last_transition_origin.as_deref(),
        Some("cli")
    );
    assert_eq!(connected_session.owner_kind.as_deref(), Some("cli"));

    let disconnected = RuntimeService::new(&context)
        .disconnect()
        .await
        .expect("disconnect should succeed");
    assert!(disconnected.stopped_session);

    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest session should load")
        .expect("latest session should exist");
    assert_eq!(latest.id, connected_session.id);
    assert_eq!(latest.status, RuntimeSessionStatus::Stopped);
    assert_eq!(
        latest.last_transition_reason_code.as_deref(),
        Some("manual_disconnect")
    );
    assert_eq!(
        latest.last_transition_reason_detail.as_deref(),
        Some("runtime disconnect request succeeded")
    );
    assert_eq!(latest.last_transition_origin.as_deref(), Some("cli"));
    assert_eq!(latest.owner_kind.as_deref(), Some("cli"));

    let _ = xray_runtime::terminate_process_gracefully(connected.pid as i64, SHUTDOWN_TIMEOUT);
}
