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
    assert_eq!(app.status_message, "tested 3 configs");
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
    assert_eq!(app.status_message, "TestBatch cancelled");
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
    assert_eq!(app.status_message, "reloaded");
    assert_eq!(app.data.total_configs, 2);
}
