use super::prelude::*;
use super::settings::{
    settings_range_pair, settings_section_name, settings_section_tree_label,
    settings_value_display, settings_value_group, settings_value_with_unit,
};

use std::fs;

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use super::*;
use crate::app::config::ConfigEditSession;
use crate::tui::app::{
    RenameModalState, SettingsEditState, SettingsModalState, SettingsPane, TuiAction,
};

#[test]
fn rename_modal_identifies_subscription_by_ref_and_name() {
    let app = TuiApp {
        rename_modal: Some(RenameModalState {
            source_id: xrat_model::SubscriptionId(7),
            source_ref: "sub-a1b2c3".to_string(),
            current_name: "Primary".to_string(),
            input: "Primary".to_string(),
            error: None,
        }),
        ..TuiApp::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_rename_modal(frame, frame.area(), &app))
        .unwrap();

    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains("Rename sub-a1b2c3 · Primary"));
    assert!(rendered.contains("Enter save   Esc cancel"));
    assert!(!rendered.contains("New subscription name"));
    assert!(!rendered.contains("Subscription #7"));
}

#[test]
fn settings_modal_never_renders_secret_plaintext() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(&path, "[server]\nkey = \"top-secret\"\n").expect("config should be written");
    let session = ConfigEditSession::open(&path).expect("settings should open");
    let secret_index = session
        .settings
        .iter()
        .position(|setting| setting.path == "server.key")
        .expect("secret setting should exist");
    let mut modal = SettingsModalState::new(session);
    modal.section_index = modal
        .sections()
        .iter()
        .position(|section| section == "server")
        .expect("server section should exist");
    modal.field_index = modal
        .visible_setting_indices()
        .iter()
        .position(|index| *index == secret_index)
        .expect("secret field should be visible");
    modal.editing = Some(SettingsEditState {
        setting_index: secret_index,
        input: "replacement".to_string(),
    });
    let app = TuiApp {
        settings_modal: Some(modal),
        ..TuiApp::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(110, 34)).unwrap();

    terminal
        .draw(|frame| render_settings_modal(frame, frame.area(), &app))
        .unwrap();

    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(!rendered.contains("top-secret"));
    assert!(!rendered.contains("replacement"));
    assert!(rendered.contains("configured"));
    assert!(rendered.contains("XRAT_API_KEY"));
}

#[test]
fn settings_sections_render_as_capitalized_tree_rows() {
    let sections = [
        "runtime",
        "runtime.http",
        "runtime.socks",
        "runtime.stats",
        "testing",
        "testing.download",
    ]
    .map(str::to_string);

    let labels: Vec<String> = sections
        .iter()
        .map(|section| settings_section_tree_label(section, &sections))
        .collect();

    assert_eq!(
        labels,
        [
            "Runtime",
            "├─ HTTP",
            "├─ Socks",
            "└─ Stats",
            "Testing",
            "└─ Download",
        ]
    );
    assert_eq!(
        settings_value_group("runtime.socks", "runtime.socks"),
        "General"
    );
    assert_eq!(
        settings_value_group("runtime.socks", "runtime.socks.auth"),
        "Authentication"
    );
    assert_eq!(settings_section_name("dns"), "DNS");
    assert_eq!(settings_section_tree_label("dns", &sections), "DNS");
    assert_eq!(
        settings_value_display(
            "runtime.sniffing.domains_excluded",
            &SettingValue::List(Vec::new()),
            false
        ),
        "none"
    );
    assert_eq!(
        settings_value_display("testing.concurrency", &SettingValue::Integer(0), false),
        "auto"
    );
    assert_eq!(
        settings_value_display(
            "runtime.rotation.test_concurrency",
            &SettingValue::Integer(0),
            false,
        ),
        "auto"
    );
    assert_eq!(
        settings_value_display(
            "runtime.mux.xudp_concurrency",
            &SettingValue::Integer(0),
            false,
        ),
        "0"
    );
    assert_eq!(
        settings_value_display(
            "testing.real_delay.timeout",
            &SettingValue::Integer(10_000),
            false,
        ),
        "10000 ms"
    );
    assert_eq!(
        settings_value_display(
            "runtime.rotation.interval_secs",
            &SettingValue::Integer(1800),
            false,
        ),
        "1800 s"
    );
    assert_eq!(
        settings_value_display(
            "subscriptions.refresh_interval_hours",
            &SettingValue::Integer(24),
            false,
        ),
        "24 h"
    );
    assert_eq!(
        settings_value_display("server.port", &SettingValue::Integer(18203), false),
        "18203"
    );
}

#[test]
fn compact_settings_modal_shows_only_the_focused_pane() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(&path, "").expect("config should be written");
    let session = ConfigEditSession::open(&path).expect("settings should open");
    let mut app = TuiApp {
        settings_modal: Some(SettingsModalState::new(session)),
        ..TuiApp::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(60, 30)).unwrap();

    terminal
        .draw(|frame| render_settings_modal(frame, frame.area(), &app))
        .unwrap();
    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains("Sections ·"));
    assert!(!rendered.contains("Values ·"));
    assert!(rendered.contains("^S save"));
    assert!(rendered.contains("Esc close"));

    app.settings_modal.as_mut().expect("modal").pane = SettingsPane::Fields;
    terminal
        .draw(|frame| render_settings_modal(frame, frame.area(), &app))
        .unwrap();
    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(!rendered.contains("Sections ·"));
    assert!(rendered.contains("Values ·"));
}

#[test]
fn settings_section_selection_remains_visible_in_short_modal() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(&path, "").expect("config should be written");
    let session = ConfigEditSession::open(&path).expect("settings should open");
    let mut modal = SettingsModalState::new(session);
    let sections = modal.sections();
    modal.section_index = sections.len().saturating_sub(1);
    let selected = modal.selected_section().expect("selected section");
    let selected_label = settings_section_tree_label(&selected, &sections);
    let app = TuiApp {
        settings_modal: Some(modal),
        ..TuiApp::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(110, 22)).unwrap();

    terminal
        .draw(|frame| render_settings_modal(frame, frame.area(), &app))
        .unwrap();
    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains(&format!("› {selected_label}")));
}

#[test]
fn settings_help_shows_origin_default_and_dns_effect() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(&path, "[dns]\nquery_strategy = \"UseSystem\"\n").expect("config should be written");
    let session = ConfigEditSession::open(&path).expect("settings should open");
    let mut modal = SettingsModalState::new(session);
    modal.section_index = modal
        .sections()
        .iter()
        .position(|section| section == "dns")
        .expect("DNS section");
    modal.field_index = modal
        .visible_setting_indices()
        .iter()
        .position(|index| modal.session.settings[*index].path == "dns.query_strategy")
        .expect("query strategy setting");
    modal.pane = SettingsPane::Fields;
    let app = TuiApp {
        settings_modal: Some(modal),
        ..TuiApp::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(110, 34)).unwrap();

    terminal
        .draw(|frame| render_settings_modal(frame, frame.area(), &app))
        .unwrap();
    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains("DNS"));
    assert!(rendered.contains("Default"));
    assert!(rendered.contains("Source"));
    assert!(rendered.contains("Legend"));
    assert!(rendered.contains("+ explicit override"));
    assert!(rendered.contains("explicit override"));
    assert!(rendered.contains("runtime restart"));
}

#[test]
fn settings_parent_page_renders_nested_value_subheaders() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(
        &path,
        "[runtime.socks]\nenabled = true\n[runtime.socks.auth]\nenabled = false\n",
    )
    .expect("config should be written");
    let session = ConfigEditSession::open(&path).expect("settings should open");
    let mut modal = SettingsModalState::new(session);
    modal.section_index = modal
        .sections()
        .iter()
        .position(|section| section == "runtime.socks")
        .expect("runtime.socks section should exist");
    let app = TuiApp {
        settings_modal: Some(modal),
        ..TuiApp::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(110, 34)).unwrap();

    terminal
        .draw(|frame| render_settings_modal(frame, frame.area(), &app))
        .unwrap();

    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains("General"));
    assert!(rendered.contains("Authentication"));
    assert!(rendered.contains('✓'));
    assert!(rendered.contains('✗'));
    assert!(!rendered.contains("runtime.socks.auth"));
}

#[test]
fn settings_help_follows_selection_in_compact_terminal() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(&path, "[runtime.socks]\nenabled = true\nport = 18200\n")
        .expect("config should be written");
    let session = ConfigEditSession::open(&path).expect("settings should open");
    let mut modal = SettingsModalState::new(session);
    modal.section_index = modal
        .sections()
        .iter()
        .position(|section| section == "runtime.socks")
        .expect("runtime.socks section should exist");
    let port_index = modal
        .visible_setting_indices()
        .iter()
        .position(|index| modal.session.settings[*index].path == "runtime.socks.port")
        .expect("port setting should be visible");
    let enabled_index = modal
        .visible_setting_indices()
        .iter()
        .position(|index| modal.session.settings[*index].path == "runtime.socks.enabled")
        .expect("enabled setting should be visible");
    modal.field_index = port_index;
    modal.pane = SettingsPane::Fields;
    let mut app = TuiApp {
        settings_modal: Some(modal),
        ..TuiApp::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_settings_modal(frame, frame.area(), &app))
        .unwrap();
    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains("Help · runtime.socks.port"));
    assert!(rendered.contains("Description"));
    assert!(rendered.contains("Values"));
    assert!(rendered.contains("Example"));
    assert!(rendered.contains("port = 18200"));
    assert!(rendered.contains("runtime restart"));

    let direction = if enabled_index < port_index { -1 } else { 1 };
    for _ in 0..enabled_index.abs_diff(port_index) {
        app.apply(TuiAction::SettingsMove(direction));
    }
    terminal
        .draw(|frame| render_settings_modal(frame, frame.area(), &app))
        .unwrap();
    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains("Help · runtime.socks.enabled"));
    assert!(rendered.contains("✓ enabled · ✗ disabled"));
    assert!(rendered.contains("enabled = true"));
}

#[test]
fn settings_fragment_ranges_render_as_indented_min_max_rows() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(
        &path,
        "[runtime.fragment]\npackets = [1, 3]\nlength = [100, 200]\ninterval = [10, 20]\n",
    )
    .expect("config should be written");
    let session = ConfigEditSession::open(&path).expect("settings should open");
    let mut modal = SettingsModalState::new(session);
    modal.section_index = modal
        .sections()
        .iter()
        .position(|section| section == "runtime.fragment")
        .expect("runtime.fragment section should exist");
    let app = TuiApp {
        settings_modal: Some(modal),
        ..TuiApp::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(110, 34)).unwrap();

    terminal
        .draw(|frame| render_settings_modal(frame, frame.area(), &app))
        .unwrap();

    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.matches("min").count() >= 3);
    assert!(rendered.matches("max").count() >= 3);
    assert!(!rendered.contains("10, 20"));
    assert_eq!(
        settings_range_pair(
            "runtime.fragment.interval",
            &SettingValue::List(vec!["10".to_string(), "20".to_string()]),
        ),
        Some(("10", "20"))
    );
    assert_eq!(
        settings_value_with_unit("runtime.fragment.interval", "10"),
        "10 ms"
    );
}

#[test]
fn split_modal_renders_and_supports_mode_and_list_transitions() {
    let tun = crate::app::config::TunSettings {
        enabled: true,
        split_mode: crate::app::config::TunSplitMode::Blacklist,
        blacklist: vec!["firefox".to_string(), "/opt/discord/".to_string()],
        whitelist: vec!["telegram-desktop".to_string()],
        ..Default::default()
    };

    let mut modal = crate::tui::app::SplitModalState::from_tun(&tun);
    modal.discovered_apps = vec![crate::app::services::split_tunnel::DiscoveredApp {
        name: "Firefox Web Browser".to_string(),
        desktop_id: Some("firefox.desktop".to_string()),
        process_name: "firefox".to_string(),
        exec_path: Some("/usr/bin/firefox".to_string()),
        rule_entry: "firefox".to_string(),
        source: crate::app::services::split_tunnel::AppSource::RunningAndDesktop,
    }];

    let mut app = TuiApp {
        split_modal: Some(modal),
        ..TuiApp::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(110, 30)).unwrap();

    terminal
        .draw(|frame| render_split_modal(frame, frame.area(), &app))
        .unwrap();
    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(rendered.contains("TUN Split Tunneling"));
    assert!(rendered.contains("blacklist (2)"));
    assert!(rendered.contains("Firefox Web Browser"));

    app.apply(TuiAction::SplitCycleMode);
    assert_eq!(
        app.split_modal.as_ref().map(|m| m.split_mode),
        Some(crate::app::config::TunSplitMode::Whitelist)
    );
    assert_eq!(
        app.split_modal.as_ref().map(|m| m.list_tab),
        Some(crate::tui::app::SplitListTab::Whitelist)
    );
    assert!(app.split_modal.as_ref().is_some_and(|m| m.is_dirty()));
}
