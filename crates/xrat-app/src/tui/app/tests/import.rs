use crate::tui::app::{ImportModalState, TuiAction, TuiApp};

#[test]
fn import_modal_accepts_typed_and_pasted_text() {
    let mut app = TuiApp {
        import_modal: Some(ImportModalState::default()),
        ..TuiApp::default()
    };

    app.apply(TuiAction::ImportInput('v'));
    app.append_import_text("less://example");

    assert_eq!(
        app.import_modal.as_ref().expect("modal").input,
        "vless://example"
    );
}

#[test]
fn import_modal_filters_pua_and_control_characters() {
    let mut app = TuiApp {
        import_modal: Some(ImportModalState::default()),
        ..TuiApp::default()
    };

    // Stray PUA code point \u{e062} (Left Control) and null/control byte
    app.apply(TuiAction::ImportInput('\u{e062}'));
    app.apply(TuiAction::ImportInput('\0'));
    app.append_import_text("\u{e062}https://example.com/sub\x07");

    assert_eq!(
        app.import_modal.as_ref().expect("modal").input,
        "https://example.com/sub"
    );
}

#[test]
fn back_closes_import_modal() {
    let mut app = TuiApp {
        import_modal: Some(ImportModalState::default()),
        ..TuiApp::default()
    };

    app.apply(TuiAction::Back);

    assert!(app.import_modal.is_none());
}
