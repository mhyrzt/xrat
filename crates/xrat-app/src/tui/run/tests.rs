use super::{prepare_import_submission, tasks};
use crate::tui::app::{ImportModalState, ImportModalStep, TuiApp};

fn app_with_import_input(input: &str) -> TuiApp {
    TuiApp {
        import_modal: Some(ImportModalState {
            input: input.to_string(),
            ..ImportModalState::default()
        }),
        ..TuiApp::default()
    }
}

#[test]
fn valid_config_link_prepares_single_config_import() {
    let mut app = app_with_import_input("vless://uuid-123@example.com:443#One");

    let Some(tasks::TuiImport::Config { node, .. }) = prepare_import_submission(&mut app) else {
        panic!("config import should be prepared");
    };

    assert_eq!(node.name.as_deref(), Some("One"));
}

#[test]
fn subscription_url_transitions_to_name_step() {
    let mut app = app_with_import_input("https://example.com/sub");

    assert!(prepare_import_submission(&mut app).is_none());

    let modal = app.import_modal.expect("name modal should remain open");
    let ImportModalStep::SubscriptionName {
        url,
        suggested_name,
    } = modal.step
    else {
        panic!("subscription name step should be active");
    };
    assert_eq!(url, "https://example.com/sub");
    assert!(suggested_name.starts_with("sub-"));
    assert_eq!(suggested_name.len(), "sub-".len() + 6);
    assert!(modal.input.is_empty());
}

#[test]
fn blank_subscription_name_uses_generated_suggestion() {
    let mut app = app_with_import_input("https://example.com/sub");
    assert!(prepare_import_submission(&mut app).is_none());
    let suggested_name = match &app.import_modal.as_ref().expect("modal").step {
        ImportModalStep::SubscriptionName { suggested_name, .. } => suggested_name.clone(),
        ImportModalStep::Link => panic!("name step should be active"),
    };
    app.import_modal.as_mut().expect("modal").input = "   ".to_string();

    let Some(tasks::TuiImport::Subscription {
        url,
        name,
        fallback_name,
    }) = prepare_import_submission(&mut app)
    else {
        panic!("subscription import should be prepared");
    };

    assert_eq!(url, "https://example.com/sub");
    assert_eq!(name, None);
    assert_eq!(fallback_name, suggested_name);
}

#[test]
fn pasted_xray_json_transitions_to_name_step_without_trimming_contents() {
    let original = "  {\"outbounds\": []}\n";
    let mut app = app_with_import_input(original);
    assert!(prepare_import_submission(&mut app).is_none());
    let ImportModalStep::SubscriptionName { url, .. } = &app.import_modal.as_ref().unwrap().step
    else {
        panic!("expected name step")
    };
    assert_eq!(url, original);
}

#[test]
fn subscription_url_with_fragment_extracts_name_in_modal() {
    let mut app = app_with_import_input("https://example.com/sub#MyProvider");
    assert!(prepare_import_submission(&mut app).is_none());

    let modal = app.import_modal.as_ref().expect("modal should remain open");
    let ImportModalStep::SubscriptionName {
        url,
        suggested_name,
    } = &modal.step
    else {
        panic!("subscription name step should be active");
    };
    assert_eq!(url, "https://example.com/sub#MyProvider");
    assert_eq!(suggested_name, "MyProvider");

    // With blank input, it should use the extracted fragment name
    let Some(tasks::TuiImport::Subscription {
        name,
        fallback_name,
        ..
    }) = prepare_import_submission(&mut app)
    else {
        panic!("subscription import should be prepared");
    };
    assert_eq!(name, None);
    assert_eq!(fallback_name, "MyProvider");
}

#[test]
fn subscription_url_with_encoded_cyrillic_fragment_extracts_name() {
    let mut app = app_with_import_input(
        "https://example.com/sub#%D0%9C%D0%BE%D1%8F%20%D0%9F%D0%BE%D0%B4%D0%BF%D0%B8%D1%81%D0%BA%D0%B0",
    );
    assert!(prepare_import_submission(&mut app).is_none());

    let modal = app.import_modal.as_ref().expect("modal should remain open");
    let ImportModalStep::SubscriptionName { suggested_name, .. } = &modal.step else {
        panic!("subscription name step should be active");
    };
    assert_eq!(suggested_name, "Моя Подписка");

    let Some(tasks::TuiImport::Subscription {
        name,
        fallback_name,
        ..
    }) = prepare_import_submission(&mut app)
    else {
        panic!("subscription import should be prepared");
    };
    assert_eq!(name, None);
    assert_eq!(fallback_name, "Моя Подписка");
}

#[test]
fn subscription_url_with_query_name_extracts_name() {
    let mut app = app_with_import_input("https://example.com/sub?token=abc&name=FastVPN");
    assert!(prepare_import_submission(&mut app).is_none());

    let modal = app.import_modal.as_ref().expect("modal should remain open");
    let ImportModalStep::SubscriptionName { suggested_name, .. } = &modal.step else {
        panic!("subscription name step should be active");
    };
    assert_eq!(suggested_name, "FastVPN");
}

#[test]
fn invalid_or_multi_link_input_stays_in_modal_with_error() {
    for input in [
        "not a link",
        "http://",
        "vless://uuid-1@example.com:443#One\nvless://uuid-2@example.com:443#Two",
    ] {
        let mut app = app_with_import_input(input);

        assert!(prepare_import_submission(&mut app).is_none());
        assert!(
            app.import_modal
                .as_ref()
                .and_then(|modal| modal.error.as_ref())
                .is_some()
        );
    }
}

#[test]
fn release_note_dismissal_closes_modal_even_when_cache_save_fails() {
    use crate::app::services::release_notes::ReleaseNotes;
    use crate::tui::view::release_notes::ReleaseNotesModal;

    let root = tempfile::tempdir().unwrap();
    let notes = ReleaseNotes {
        version: "v1.2.3".into(),
        body: "# Changes".into(),
        seen: false,
    };
    notes.save(root.path()).unwrap();
    let blocked_root = tempfile::NamedTempFile::new().unwrap();
    let mut app = TuiApp::default();
    let mut modal = Some(ReleaseNotesModal::new(notes.clone()));
    super::event_loop::dismiss_release_notes(&mut app, &mut modal, blocked_root.path());
    assert!(modal.is_none());
    assert!(ReleaseNotes::pending(root.path(), "1.2.3").is_some());

    let mut modal = Some(ReleaseNotesModal::new(notes));
    super::event_loop::dismiss_release_notes(&mut app, &mut modal, root.path());
    assert!(modal.is_none());
    assert!(ReleaseNotes::pending(root.path(), "1.2.3").is_none());
}
