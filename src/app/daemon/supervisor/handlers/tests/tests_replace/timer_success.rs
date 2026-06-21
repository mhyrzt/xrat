use super::super::super::test_support::{test_context, test_node, test_source};
use super::helpers::{set_running_session, spawn_sleep, write_fake_runtime_script};
use super::*;
use crate::app::daemon::supervisor::{SupervisorEvent, SupervisorState};
use crate::db::ConnectionTestInsert;

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
            dial_endpoint_ip: None,
            dial_endpoint_location: None,
            dial_endpoint_country: None,
            dial_endpoint_asn: None,
            dial_endpoint_geoip_source: None,
            dial_endpoint_fronting: None,
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
        Some(crate::app::daemon::ipc::RotationTrigger::Timer)
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
    assert_eq!(state.last_candidate_config_id, running.config_id);
    assert_eq!(
        running.last_transition_reason_code.as_deref(),
        Some("replace_commit_success")
    );
    assert_ne!(running.process_id, Some(old_pid));

    let _ = crate::xray::process_mgmt::terminate_process_gracefully(
        running.process_id.unwrap_or_default(),
        std::time::Duration::from_millis(1500),
    );
    let _ = old.kill();
    let _ = old.wait();
}
