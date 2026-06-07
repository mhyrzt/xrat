use super::helpers::row;
use crate::tui::app::TuiApp;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

#[test]
fn completed_event_clears_cancellation_token() {
    let mut app = TuiApp::with_data(TuiData::from_configs(vec![row(1)]));
    let (_token, _receiver) = app.task_state.start(TuiTaskKind::TestBatch);

    app.apply_task_event(TuiTaskEvent::Completed {
        kind: TuiTaskKind::TestBatch,
        message: "tested 3 configs".to_string(),
        data: None,
    });

    assert!(app.task_state.cancellation.is_none());
    assert_eq!(app.task_state.running, None);
    assert_eq!(
        app.event_log.last().map(String::as_str),
        Some("OK  tested 3 configs")
    );
}

#[test]
fn cancelled_event_clears_cancellation_token() {
    let mut app = TuiApp::with_data(TuiData::from_configs(vec![row(1)]));
    let (_token, _receiver) = app.task_state.start(TuiTaskKind::TestBatch);

    app.apply_task_event(TuiTaskEvent::Cancelled {
        kind: TuiTaskKind::TestBatch,
    });

    assert!(app.task_state.cancellation.is_none());
    assert_eq!(app.task_state.running, None);
    assert_eq!(
        app.event_log.last().map(String::as_str),
        Some("OK  TestBatch cancelled")
    );
}

#[test]
fn spinner_advances_only_while_runtime_op_in_flight() {
    let mut app = TuiApp::with_data(TuiData::from_configs(vec![row(1)]));
    assert!(!app.runtime_op_in_flight());

    let before = app.spinner_frame();
    app.tick();
    assert_eq!(
        app.spinner_frame(),
        before,
        "spinner should not advance while idle"
    );

    let (_token, _receiver) = app.task_state.start(TuiTaskKind::RuntimeOp);
    assert!(app.runtime_op_in_flight());
    let frame_before = app.spinner_frame();
    app.tick();
    assert_ne!(
        app.spinner_frame(),
        frame_before,
        "spinner should advance during a runtime op"
    );
}

#[test]
fn runtime_op_completion_stops_spinner() {
    let mut app = TuiApp::with_data(TuiData::from_configs(vec![row(1)]));
    let (_token, _receiver) = app.task_state.start(TuiTaskKind::RuntimeOp);
    app.apply_task_event(TuiTaskEvent::Completed {
        kind: TuiTaskKind::RuntimeOp,
        message: "connected".to_string(),
        data: None,
    });
    assert!(!app.runtime_op_in_flight());

    let frame_before = app.spinner_frame();
    app.tick();
    assert_eq!(
        app.spinner_frame(),
        frame_before,
        "spinner should stop after the runtime op completes"
    );
}

#[test]
fn applies_task_completion_and_reloads_data() {
    let mut app = TuiApp::with_data(TuiData::from_configs(vec![row(1)]));

    app.apply_task_event(TuiTaskEvent::Started {
        kind: TuiTaskKind::ReloadData,
    });
    assert_eq!(app.task_state.running, Some(TuiTaskKind::ReloadData));

    app.apply_task_event(TuiTaskEvent::Completed {
        kind: TuiTaskKind::ReloadData,
        message: "reloaded".to_string(),
        data: Some(TuiData::from_configs(vec![row(1), row(2)])),
    });

    assert_eq!(app.task_state.running, None);
    assert_eq!(
        app.event_log.last().map(String::as_str),
        Some("OK  reloaded")
    );
    assert_eq!(app.data.total_configs, 2);
}

#[test]
fn applies_completed_config_test_row_during_batch() {
    let mut updated = row(2);
    updated.real_delay_ms = Some(50);

    let mut app = TuiApp::with_data(TuiData::from_configs(vec![row(1), row(2)]));
    let (_token, _receiver) = app.task_state.start(TuiTaskKind::TestBatch);
    app.testing_config_ids = vec![1, 2];
    app.config_list.focused = 1;

    app.apply_task_event(TuiTaskEvent::ConfigTested {
        row: updated,
        done: 1,
        total: 2,
    });

    assert_eq!(app.task_state.progress_done, 1);
    assert_eq!(app.task_state.progress_total, 2);
    assert_eq!(app.testing_config_ids, vec![1]);
    assert_eq!(
        app.data
            .configs
            .iter()
            .find(|config| config.id == 2)
            .and_then(|config| config.real_delay_ms),
        Some(50)
    );
    assert_eq!(app.focused_config().map(|config| config.id), Some(2));
}
