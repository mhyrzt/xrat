use super::{TuiTaskEvent, TuiTaskKind, TuiTaskState};

#[test]
fn tracks_task_lifecycle() {
    let mut state = TuiTaskState::default();

    state.apply(&TuiTaskEvent::Started {
        kind: TuiTaskKind::ReloadData,
    });
    assert_eq!(state.running, Some(TuiTaskKind::ReloadData));
    assert_eq!(state.label(), "ReloadData running");

    state.apply(&TuiTaskEvent::Completed {
        kind: TuiTaskKind::ReloadData,
        message: "reloaded".to_string(),
        data: None,
    });

    assert_eq!(state.running, None);
    assert_eq!(state.completed_count, 1);
    assert_eq!(state.label(), "reloaded");
}

#[test]
fn source_refresh_label_uses_subscriptions_wording() {
    let mut state = TuiTaskState::default();
    state.apply(&TuiTaskEvent::Started {
        kind: TuiTaskKind::SourceRefresh,
    });
    assert_eq!(state.label(), "Subscriptions refreshing");
}

#[test]
fn import_label_is_concise() {
    let mut state = TuiTaskState::default();
    state.apply(&TuiTaskEvent::Started {
        kind: TuiTaskKind::Import,
    });
    assert_eq!(state.label(), "Importing");
}

#[test]
fn start_creates_cancellation_token() {
    let mut state = TuiTaskState::default();
    let (token, _receiver) = state.start(TuiTaskKind::TestBatch);

    assert!(!token.is_cancelled());
    assert_eq!(state.running, Some(TuiTaskKind::TestBatch));
}

#[test]
fn cancel_only_succeeds_when_token_present() {
    let mut state = TuiTaskState::default();
    assert!(!state.cancel());

    let (_token, _receiver) = state.start(TuiTaskKind::TestBatch);
    assert!(state.cancel());
}

#[test]
fn cancellation_clears_token() {
    let mut state = TuiTaskState::default();
    let _token = state.start(TuiTaskKind::TestBatch);

    state.apply(&TuiTaskEvent::Cancelled {
        kind: TuiTaskKind::TestBatch,
    });

    assert!(state.cancellation.is_none());
    assert_eq!(state.label(), "TestBatch cancelled");
}
