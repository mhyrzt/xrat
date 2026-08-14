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

    let Some(tasks::TuiImport::Subscription { url, name }) = prepare_import_submission(&mut app)
    else {
        panic!("subscription import should be prepared");
    };

    assert_eq!(url, "https://example.com/sub");
    assert_eq!(name, suggested_name);
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
