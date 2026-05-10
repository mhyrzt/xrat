use super::super::reattach::ProcessInspector;
use super::super::*;
use super::test_support::{test_context, test_node, test_source};

#[tokio::test]
async fn reattach_rejects_pid_missing_marks_session_failed() {
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

    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should be set");
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
            process_id: None,
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert");

    RuntimeService::new(&context)
        .reconcile_reattach_on_daemon_start()
        .await
        .expect("reattach reconcile should succeed");

    let session = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_eq!(session.status, RuntimeSessionStatus::Failed);
    assert_eq!(
        session.failure_reason.as_deref(),
        Some("daemon_restart_reattach_rejected_pid_missing")
    );
    assert!(
        context
            .db
            .get_active_config()
            .await
            .expect("active should load")
            .is_none()
    );
}

#[tokio::test]
async fn reattach_rejects_exec_mismatch_marks_session_failed() {
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

    // Force exec mismatch by expecting a binary name that cannot match this test process.
    context.runtime_paths.xray_path = std::path::PathBuf::from("definitely-not-this-binary-name");

    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should be set");
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
            process_id: Some(i64::from(std::process::id())),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert");

    RuntimeService::new(&context)
        .reconcile_reattach_on_daemon_start()
        .await
        .expect("reattach reconcile should succeed");

    let session = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_eq!(session.status, RuntimeSessionStatus::Failed);
    assert_eq!(
        session.failure_reason.as_deref(),
        Some("daemon_restart_reattach_rejected_exec_mismatch")
    );
    assert!(
        context
            .db
            .get_active_config()
            .await
            .expect("active should load")
            .is_none()
    );
}

struct AcceptingInspector;

impl ProcessInspector for AcceptingInspector {
    fn is_running(&self, _pid: i64) -> bool {
        true
    }

    fn exec_matches_runtime_engine(&self, _context: &AppContext, _pid: i64) -> bool {
        true
    }

    fn cmdline_matches_session_config(
        &self,
        _context: &AppContext,
        _pid: i64,
        _session_id: i64,
    ) -> bool {
        true
    }
}

#[tokio::test]
async fn reattach_accepts_matching_pid_exec_cmdline() {
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
            process_id: Some(12345),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert");

    RuntimeService::new(&context)
        .reconcile_reattach_with_inspector(&AcceptingInspector)
        .await
        .expect("reattach reconcile should succeed");

    let session = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_eq!(session.id, session_id);
    assert_eq!(session.status, RuntimeSessionStatus::Running);
    assert!(session.failure_reason.is_none());
    assert_eq!(
        context
            .db
            .get_active_config()
            .await
            .expect("active should load")
            .map(|row| row.id),
        Some(config.id)
    );
}
