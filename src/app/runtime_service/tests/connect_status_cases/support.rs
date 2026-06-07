use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

pub(super) async fn import_single_config(context: &AppContext) -> ConfigRecord {
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
        .await
        .expect("node should import");
    assert_eq!(summary.imported_configs, 1);

    context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist")
}

pub(super) async fn import_hy2_config(context: &AppContext) -> ConfigRecord {
    let node = crate::model::Node {
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
    };
    let summary = context
        .db
        .import_nodes(&test_source(), &[node])
        .await
        .expect("node should import");
    assert_eq!(summary.imported_configs, 1);

    context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist")
}

pub(super) fn write_fake_runtime_script(context: &AppContext) {
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
port = int(inbound.get("port", inbound.get("listen_port")))
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
