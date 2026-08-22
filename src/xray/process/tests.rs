use super::*;
use crate::xray::XrayConfig;
use crate::xray::config::{Inbound, LogConfig, Outbound};
use std::process::Command;
use std::time::Duration;
use tokio::net::TcpStream;

#[tokio::test]
async fn test_xray_process_lifecycle() {
    // This test requires xray to be installed
    // Skip if xray is not available
    if Command::new("xray").arg("version").output().is_err() {
        tracing::warn!("skipping test: xray not found");
        return;
    }

    let config = XrayConfig {
        log: LogConfig {
            loglevel: "warning".to_string(),
        },
        inbounds: vec![Inbound {
            tag: "test-in".to_string(),
            port: 10809,
            listen: "127.0.0.1".to_string(),
            protocol: "socks".to_string(),
            settings: Some(serde_json::json!({"udp": false})),
        }],
        outbounds: vec![Outbound {
            tag: "direct".to_string(),
            protocol: "freedom".to_string(),
            settings: serde_json::json!({}),
            stream_settings: None,
            mux: None,
        }],
        dns: None,
        api: None,
        stats: None,
        policy: None,
        routing: None,
    };

    let process = XrayProcess::spawn(&config, Duration::from_secs(5)).await;

    let process = match process {
        Ok(process) => process,
        Err(XrayProcessError::ProcessExited(_))
        | Err(XrayProcessError::PortNotReady(_))
        | Err(XrayProcessError::SpawnError(_)) => return,
        Err(error) => panic!("Failed to spawn xray: {error}"),
    };

    assert!(process.pid() > 0);
    assert_eq!(process.local_port(), 10809);

    // Verify port is listening
    let conn = TcpStream::connect("127.0.0.1:10809").await;
    assert!(conn.is_ok());

    // Clean up
    process.kill().expect("Failed to kill process");
}
