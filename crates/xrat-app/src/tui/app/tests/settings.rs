use std::fs;

use crate::app::config::ConfigEditSession;
use crate::tui::app::{SettingsModalState, SettingsPane, TuiAction, TuiApp};

#[test]
fn tun_setting_is_editable_and_persisted_from_settings() {
    let _guard = crate::tui::theme::lock_test_theme();
    let (_root, mut app) = app_with_settings("");
    focus_setting(&mut app, "runtime.tun.enabled");
    app.apply(TuiAction::SettingsSubmit);
    assert!(app.prepare_settings_save());
    let outcome = app.settings_modal.as_mut().unwrap().session.save().unwrap();
    assert!(outcome.config.runtime.tun.enabled);
    assert_eq!(outcome.changed_paths, ["runtime.tun.enabled"]);
}

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
    let _guard = crate::tui::theme::lock_test_theme();
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
    let _guard = crate::tui::theme::lock_test_theme();
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
    let _guard = crate::tui::theme::lock_test_theme();
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
    let _guard = crate::tui::theme::lock_test_theme();
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
    let _guard = crate::tui::theme::lock_test_theme();
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
    let _guard = crate::tui::theme::lock_test_theme();
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

#[test]
fn theme_setting_cycles_previews_live_and_reverts_on_discard() {
    let _guard = crate::tui::theme::lock_test_theme();
    crate::tui::theme::set_active(crate::tui::theme::TuiTheme::Default);
    let (_root, mut app) = app_with_settings("[tui]\ntheme = \"default\"\n");
    focus_setting(&mut app, "tui.theme");

    app.apply(TuiAction::SettingsCycle(1));
    assert_eq!(
        crate::tui::theme::palette(),
        crate::tui::theme::TuiTheme::Ansi.palette()
    );

    app.apply(TuiAction::Back);
    app.apply(TuiAction::SettingsConfirmDiscard(true));
    assert_eq!(
        crate::tui::theme::palette(),
        crate::tui::theme::TuiTheme::Default.palette()
    );
}

#[test]
fn theme_setting_saves_to_config() {
    let _guard = crate::tui::theme::lock_test_theme();
    crate::tui::theme::set_active(crate::tui::theme::TuiTheme::Default);
    let (_root, mut app) = app_with_settings("[tui]\ntheme = \"default\"\n");
    focus_setting(&mut app, "tui.theme");

    app.apply(TuiAction::SettingsCycle(1));
    app.apply(TuiAction::SettingsCycle(1));
    assert_eq!(
        crate::tui::theme::palette(),
        crate::tui::theme::TuiTheme::Catppuccin.palette()
    );

    assert!(app.prepare_settings_save());
    let outcome = app.settings_modal.as_mut().unwrap().session.save().unwrap();
    assert_eq!(
        outcome.config.tui.theme,
        crate::tui::theme::TuiTheme::Catppuccin.into()
    );
    assert_eq!(outcome.changed_paths, ["tui.theme"]);
}

#[test]
fn custom_theme_previews_and_discard_restores_original_custom_palette() {
    let _guard = crate::tui::theme::lock_test_theme();
    let contents = r##"
[tui]
theme = "custom"
[tui.themes.custom]
extends = "light"
accent = "#123456"
"##;
    let (_root, mut app) = app_with_settings(contents);
    let settings = &app.settings_modal.as_ref().unwrap().session.tui;
    let original = settings.resolve(&settings.theme).unwrap();
    crate::tui::theme::set_palette(original);
    focus_setting(&mut app, "tui.theme");
    app.apply(TuiAction::SettingsCycle(1));
    assert_ne!(crate::tui::theme::palette(), original);
    app.apply(TuiAction::Back);
    app.apply(TuiAction::SettingsConfirmDiscard(true));
    assert_eq!(crate::tui::theme::palette(), original);
    crate::tui::theme::set_active(crate::tui::theme::TuiTheme::Default);
}

#[test]
fn custom_theme_selection_saves_and_preserves_definition_and_comments() {
    let _guard = crate::tui::theme::lock_test_theme();
    let contents = r##"
[tui]
theme = "light"
# Keep my palette intact.
[tui.themes.custom]
extends = "nord"
accent = "#123456" # my accent
"##;
    let (root, mut app) = app_with_settings(contents);
    focus_setting(&mut app, "tui.theme");
    app.apply(TuiAction::SettingsCycle(1));
    assert_eq!(
        crate::tui::theme::palette().accent,
        ratatui::style::Color::Rgb(18, 52, 86)
    );
    assert!(app.prepare_settings_save());
    let outcome = app.settings_modal.as_mut().unwrap().session.save().unwrap();
    assert_eq!(outcome.config.tui.theme.as_str(), "custom");
    let saved = fs::read_to_string(root.path().join("config.toml")).unwrap();
    assert!(saved.contains("# Keep my palette intact."));
    assert!(saved.contains("accent = \"#123456\" # my accent"));
    assert_eq!(outcome.changed_paths, ["tui.theme"]);
    focus_setting(&mut app, "runtime.socks.enabled");
    app.apply(TuiAction::SettingsSubmit);
    let second = app.settings_modal.as_mut().unwrap().session.save().unwrap();
    assert_eq!(second.config.tui.themes, outcome.config.tui.themes);
    crate::tui::theme::set_active(crate::tui::theme::TuiTheme::Default);
}

#[test]
fn reset_theme_previews_default_and_discard_restores_custom() {
    let _guard = crate::tui::theme::lock_test_theme();
    let (_root, mut app) =
        app_with_settings("[tui]\ntheme = 'custom'\n[tui.themes.custom]\nextends = 'light'");
    focus_setting(&mut app, "tui.theme");
    app.apply(TuiAction::SettingsReset);
    assert_eq!(
        crate::tui::theme::palette(),
        crate::tui::theme::TuiTheme::Default.palette()
    );
    app.apply(TuiAction::Back);
    app.apply(TuiAction::SettingsConfirmDiscard(true));
    assert_eq!(
        crate::tui::theme::palette(),
        crate::tui::theme::TuiTheme::Light.palette()
    );
    crate::tui::theme::set_active(crate::tui::theme::TuiTheme::Default);
}

#[test]
fn discard_after_save_restores_last_saved_palette() {
    let _guard = crate::tui::theme::lock_test_theme();
    let (_root, mut app) =
        app_with_settings("[tui]\ntheme = 'light'\n[tui.themes.custom]\nextends = 'nord'\n");
    focus_setting(&mut app, "tui.theme");
    app.apply(TuiAction::SettingsCycle(1));
    assert!(app.prepare_settings_save());
    app.settings_modal.as_mut().unwrap().session.save().unwrap();
    let saved = crate::tui::theme::palette();
    app.apply(TuiAction::SettingsCycle(1));
    assert_ne!(crate::tui::theme::palette(), saved);
    app.apply(TuiAction::Back);
    app.apply(TuiAction::SettingsConfirmDiscard(true));
    assert_eq!(crate::tui::theme::palette(), saved);
    crate::tui::theme::set_active(crate::tui::theme::TuiTheme::Default);
}

#[test]
fn legacy_ui_theme_can_be_saved_without_duplicate_sections() {
    let _guard = crate::tui::theme::lock_test_theme();
    let (root, mut app) =
        app_with_settings("[ui]\ntheme = 'light'\n[ui.themes.custom]\nextends = 'nord'\n");
    focus_setting(&mut app, "tui.theme");
    app.apply(TuiAction::SettingsCycle(1));
    let outcome = app.settings_modal.as_mut().unwrap().session.save().unwrap();
    assert_eq!(outcome.config.tui.theme.as_str(), "custom");
    let saved = fs::read_to_string(root.path().join("config.toml")).unwrap();
    assert!(saved.contains("[tui.themes.custom]"));
    assert!(!saved.contains("[ui]"));
    crate::tui::theme::set_active(crate::tui::theme::TuiTheme::Default);
}

#[test]
fn settings_search_delete_word_pops_last_segment() {
    let (_root, mut app) = app_with_settings("[runtime.socks]\nenabled = true\n");
    app.apply(TuiAction::SettingsBeginSearch);
    for ch in "first second  ".chars() {
        app.apply(TuiAction::SettingsInput(ch));
    }
    assert_eq!(app.settings_modal.as_ref().unwrap().query, "first second  ");
    app.apply(TuiAction::SettingsDeleteWord);
    assert_eq!(app.settings_modal.as_ref().unwrap().query, "first ");
    app.apply(TuiAction::SettingsDeleteWord);
    assert_eq!(app.settings_modal.as_ref().unwrap().query, "");
}

#[test]
fn settings_search_esc_clears_active_query_before_closing_modal() {
    let _guard = crate::tui::theme::lock_test_theme();
    let (_root, mut app) = app_with_settings("[runtime.socks]\nenabled = true\n");
    app.apply(TuiAction::SettingsBeginSearch);
    for ch in "socks".chars() {
        app.apply(TuiAction::SettingsInput(ch));
    }
    app.apply(TuiAction::SettingsSubmit);
    assert_eq!(app.settings_modal.as_ref().unwrap().query, "socks");
    assert!(!app.settings_modal.as_ref().unwrap().searching);

    app.apply(TuiAction::Back);
    assert!(app.settings_modal.is_some());
    assert_eq!(app.settings_modal.as_ref().unwrap().query, "");

    app.apply(TuiAction::Back);
    assert!(app.settings_modal.is_none());
}
