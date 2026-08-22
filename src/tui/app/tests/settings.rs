use std::fs;

use crate::app::config::ConfigEditSession;
use crate::tui::app::{SettingsModalState, SettingsPane, TuiAction, TuiApp};

fn app_with_settings(contents: &str) -> (tempfile::TempDir, TuiApp) {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(path.as_path(), contents).expect("config should be written");
    let session = ConfigEditSession::open(path.as_path()).expect("settings should open");
    let app = TuiApp {
        settings_modal: Some(SettingsModalState::new(session)),
        ..TuiApp::default()
    };
    (root, app)
}

fn focus_setting(app: &mut TuiApp, path: &str) {
    let modal = app.settings_modal.as_mut().expect("modal should exist");
    let setting_index = modal
        .session
        .settings
        .iter()
        .position(|setting| setting.path == path)
        .expect("setting should exist");
    let section = modal.session.settings[setting_index].section.clone();
    modal.section_index = modal
        .sections()
        .iter()
        .position(|candidate| candidate == &section)
        .expect("section should exist");
    modal.field_index = modal
        .visible_setting_indices()
        .iter()
        .position(|candidate| *candidate == setting_index)
        .expect("field should exist");
    modal.pane = SettingsPane::Fields;
}

#[test]
fn enter_toggles_boolean_and_marks_modal_dirty() {
    let (_root, mut app) = app_with_settings("[runtime.socks]\nenabled = true\n");
    focus_setting(&mut app, "runtime.socks.enabled");

    app.apply(TuiAction::SettingsSubmit);

    let modal = app
        .settings_modal
        .as_ref()
        .expect("modal should remain open");
    assert!(modal.session.is_dirty());
    assert_eq!(
        modal.session.settings[modal.selected_setting_index().expect("selected setting")]
            .value
            .display(false),
        "false"
    );
}

#[test]
fn reset_is_ignored_while_sections_pane_is_focused() {
    let (_root, mut app) = app_with_settings("[runtime.socks]\nport = 1080\n");
    focus_setting(&mut app, "runtime.socks.port");
    app.settings_modal.as_mut().expect("modal").pane = SettingsPane::Sections;

    app.apply(TuiAction::SettingsReset);

    let modal = app.settings_modal.as_ref().expect("modal");
    assert!(!modal.session.is_dirty());
    assert_eq!(
        modal.session.settings[modal.selected_setting_index().expect("setting")]
            .value
            .display(false),
        "1080"
    );
}

#[test]
fn dns_settings_can_be_changed() {
    let (_root, mut app) = app_with_settings("[dns]\nquery_strategy = \"UseSystem\"\n");
    focus_setting(&mut app, "dns.query_strategy");

    app.apply(TuiAction::SettingsSubmit);

    let modal = app.settings_modal.as_ref().expect("modal");
    assert!(modal.session.is_dirty());
    assert!(modal.editing.is_none());
    assert!(modal.notice.is_none());
}

#[test]
fn save_preparation_commits_valid_edits_and_rejects_invalid_ones() {
    let (_root, mut app) = app_with_settings("[runtime.socks]\nport = 18200\n");
    focus_setting(&mut app, "runtime.socks.port");
    app.apply(TuiAction::SettingsSubmit);
    app.apply(TuiAction::SettingsClearInput);
    app.append_settings_text("1080");

    assert!(app.prepare_settings_save());
    assert!(
        app.settings_modal
            .as_ref()
            .expect("modal")
            .session
            .is_dirty()
    );

    app.apply(TuiAction::SettingsSubmit);
    app.apply(TuiAction::SettingsClearInput);
    app.append_settings_text("invalid");

    assert!(!app.prepare_settings_save());
    let modal = app.settings_modal.as_ref().expect("modal");
    assert!(modal.editing.is_some());
    assert!(modal.error.is_some());
}

#[test]
fn scalar_edit_and_discard_confirmation_are_stateful() {
    let (_root, mut app) = app_with_settings("[runtime.socks]\nport = 18200\n");
    focus_setting(&mut app, "runtime.socks.port");

    app.apply(TuiAction::SettingsSubmit);
    app.apply(TuiAction::SettingsClearInput);
    app.apply(TuiAction::SettingsInput('1'));
    app.apply(TuiAction::SettingsInput('0'));
    app.apply(TuiAction::SettingsInput('8'));
    app.apply(TuiAction::SettingsInput('0'));
    app.apply(TuiAction::SettingsSubmit);
    app.apply(TuiAction::Back);

    assert!(
        app.settings_modal
            .as_ref()
            .expect("dirty modal should remain")
            .discard_confirm
    );
    app.apply(TuiAction::SettingsConfirmDiscard(false));
    assert!(
        !app.settings_modal
            .as_ref()
            .expect("modal should remain")
            .discard_confirm
    );
    app.apply(TuiAction::Back);
    app.apply(TuiAction::SettingsConfirmDiscard(true));
    assert!(app.settings_modal.is_none());
}

#[test]
fn secret_edit_starts_empty_instead_of_revealing_value() {
    let (_root, mut app) = app_with_settings("[server]\nkey = \"top-secret\"\n");
    focus_setting(&mut app, "server.key");

    app.apply(TuiAction::SettingsSubmit);

    assert_eq!(
        app.settings_modal
            .as_ref()
            .and_then(|modal| modal.editing.as_ref())
            .map(|editing| editing.input.as_str()),
        Some("")
    );
}

#[test]
fn values_pane_has_independent_vertical_navigation() {
    let (_root, mut app) = app_with_settings("[runtime.socks]\nenabled = true\n");
    let modal = app.settings_modal.as_mut().expect("modal should exist");
    modal.section_index = modal
        .sections()
        .iter()
        .position(|section| section == "runtime.socks")
        .expect("runtime.socks section should exist");

    app.apply(TuiAction::SettingsFocusFields);
    app.apply(TuiAction::SettingsMove(1));

    let modal = app.settings_modal.as_ref().expect("modal should remain");
    assert_eq!(modal.pane, SettingsPane::Fields);
    assert_eq!(modal.field_index, 1);
}

#[test]
fn enter_on_sections_focuses_values_without_editing() {
    let (_root, mut app) = app_with_settings("[runtime.socks]\nenabled = true\n");

    app.apply(TuiAction::SettingsSubmit);

    let modal = app.settings_modal.as_ref().expect("modal should remain");
    assert_eq!(modal.pane, SettingsPane::Fields);
    assert!(modal.editing.is_none());
    assert!(!modal.session.is_dirty());
}

#[test]
fn nested_groups_are_folded_into_their_parent_page() {
    let (_root, mut app) = app_with_settings("[runtime.socks.auth]\nenabled = true\n");
    let modal = app.settings_modal.as_mut().expect("modal should exist");
    assert!(
        !modal
            .sections()
            .iter()
            .any(|section| section == "runtime.socks.auth")
    );
    modal.section_index = modal
        .sections()
        .iter()
        .position(|section| section == "runtime.socks")
        .expect("runtime.socks section should exist");

    let visible_sections: Vec<&str> = modal
        .visible_setting_indices()
        .iter()
        .map(|index| modal.session.settings[*index].section.as_str())
        .collect();
    assert!(visible_sections.contains(&"runtime.socks"));
    assert!(visible_sections.contains(&"runtime.socks.auth"));
}
