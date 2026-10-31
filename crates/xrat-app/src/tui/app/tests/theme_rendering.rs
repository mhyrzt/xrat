use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

use super::helpers::{row, source};
use crate::tui::{
    app::{TuiApp, TuiView},
    data::{EngineInfo, TuiData},
    theme::{self, TuiTheme},
};

fn app() -> TuiApp {
    let mut data = TuiData::from_configs_and_sources(vec![row(1), row(2)], vec![source(1)]);
    data.metric_columns = crate::tui::data::TuiMetricColumns {
        real_delay: true,
        download: true,
        ..Default::default()
    };
    TuiApp {
        data,
        engines: vec![
            EngineInfo {
                name: "xray",
                available: true,
                version: Some("26.9.30".to_string()),
            },
            EngineInfo {
                name: "sing-box",
                available: false,
                version: None,
            },
        ],
        ..TuiApp::default()
    }
}

fn find_cell<'a>(buffer: &'a Buffer, text: &str) -> &'a ratatui::buffer::Cell {
    for y in 0..buffer.area.height {
        let symbols: Vec<_> = (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect();
        for x in 0..symbols.len() {
            if symbols[x..].concat().starts_with(text) {
                return &buffer[(x as u16, y)];
            }
        }
    }
    panic!("text {text:?} missing from rendered buffer");
}

#[test]
fn palettes_cover_screen_selections_and_mixed_engine_states() {
    let _guard = theme::TEST_THEME_LOCK.lock().unwrap();
    let mut app = app();
    let mut terminal = Terminal::new(TestBackend::new(200, 48)).unwrap();
    let custom: crate::app::config::TuiSettings = toml::from_str(
        r##"
        theme = "custom"
        [themes.custom]
        extends = "light"
        background = "#eeeeee"
        foreground = "#222222"
        selection_background = "#dddddd"
        selection_foreground = "#111111"
    "##,
    )
    .unwrap();
    let mut palettes: Vec<_> = TuiTheme::CANONICAL_NAMES
        .iter()
        .map(|name| {
            let builtin: TuiTheme = name.parse().unwrap();
            (name.to_string(), builtin.palette())
        })
        .collect();
    palettes.push(("custom".to_string(), custom.resolve(&custom.theme).unwrap()));
    for (name, palette) in palettes {
        theme::set_palette(palette);
        app.active_view = TuiView::Configs;
        terminal
            .draw(|frame| crate::tui::view::render(frame, &app))
            .unwrap();
        let buffer = terminal.backend().buffer();
        snapshot(buffer, &format!("{name}-configs"));
        assert_eq!(buffer[(0, 0)].bg, palette.background);
        assert_eq!(
            find_cell(buffer, "example.com").fg,
            palette.selection_foreground
        );
        assert_eq!(
            find_cell(buffer, "example.com").bg,
            palette.selection_background
        );
        assert_eq!(find_cell(buffer, "xray 26.9.30 ✓").fg, palette.success);
        assert_eq!(find_cell(buffer, "sing-box ✗").fg, palette.failure);
        snapshot(buffer, &format!("{name}-configs"));

        app.active_view = TuiView::Sources;
        app.source_list.focused = 1;
        terminal
            .draw(|frame| crate::tui::view::render(frame, &app))
            .unwrap();
        let selected = find_cell(terminal.backend().buffer(), "Orphans");
        assert_eq!(selected.fg, palette.selection_foreground);
        assert_eq!(selected.bg, palette.selection_background);
        snapshot(terminal.backend().buffer(), &format!("{name}-sources"));

        app.show_help = true;
        terminal
            .draw(|frame| crate::tui::view::render(frame, &app))
            .unwrap();
        snapshot(terminal.backend().buffer(), &format!("{name}-help"));
        app.show_help = false;

        let mut notes = crate::tui::view::release_notes::ReleaseNotesModal::new(
            crate::app::services::release_notes::ReleaseNotes {
                version: "0.28.0".to_string(),
                body: "# Changes\n\n- **Fixed** theme colors\n- `code` and [docs](https://example.com)".to_string(),
                seen: false,
            }
        );
        terminal.draw(|frame| notes.render(frame)).unwrap();
        let heading = find_cell(terminal.backend().buffer(), "Changes");
        assert_eq!(heading.fg, palette.accent);
        assert_eq!(heading.bg, palette.background);
        snapshot(terminal.backend().buffer(), &format!("{name}-notes"));
        // Constructed Markdown must follow subsequent palette changes too.
        theme::set_active(TuiTheme::Nord);
        terminal.draw(|frame| notes.render(frame)).unwrap();
        assert_eq!(
            find_cell(terminal.backend().buffer(), "Changes").fg,
            TuiTheme::Nord.palette().accent
        );
    }
    theme::set_active(TuiTheme::Default);
}

#[test]
fn custom_palette_is_used_in_settings_modal() {
    let _guard = theme::TEST_THEME_LOCK.lock().unwrap();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config.toml");
    std::fs::write(
        &path,
        "[tui]\ntheme = 'custom'\n[tui.themes.custom]\nextends = 'light'\n",
    )
    .unwrap();
    let session = crate::app::config::ConfigEditSession::open(&path).unwrap();
    let palette = session.tui.resolve(&session.tui.theme).unwrap();
    theme::set_palette(palette);
    let mut app = app();
    app.settings_modal = Some(crate::tui::app::SettingsModalState::new(session));
    let mut terminal = Terminal::new(TestBackend::new(200, 48)).unwrap();
    terminal
        .draw(|frame| crate::tui::view::render(frame, &app))
        .unwrap();
    let title = find_cell(terminal.backend().buffer(), "Settings");
    assert_eq!(title.fg, palette.accent);
    assert_eq!(title.bg, palette.background);
    snapshot(terminal.backend().buffer(), "light-settings");
    theme::set_active(TuiTheme::Default);
}

#[test]
fn transparent_theme_renders_reset_background_with_theme_accents() {
    let _guard = theme::TEST_THEME_LOCK.lock().unwrap();
    let app = app();
    let mut terminal = Terminal::new(TestBackend::new(200, 48)).unwrap();
    let tui: crate::app::config::TuiSettings = toml::from_str(
        r##"
        theme = "catppuccin"
        transparent = true
    "##,
    )
    .unwrap();
    let palette = tui.resolve(&tui.theme).unwrap();
    assert_eq!(palette.background, ratatui::style::Color::Reset);
    theme::set_palette(palette);
    terminal
        .draw(|frame| crate::tui::view::render(frame, &app))
        .unwrap();
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(0, 0)].bg, ratatui::style::Color::Reset);
    assert_eq!(find_cell(buffer, "xray 26.9.30 ✓").fg, palette.success);
    theme::set_active(TuiTheme::Default);
}

#[test]
fn key_bar_connection_pill_shows_running_config_connecting_and_idle() {
    let _guard = theme::TEST_THEME_LOCK.lock().unwrap();
    let mut app = app();
    let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();

    // 1. Idle state
    app.data.runtime.pid_running = false;
    app.data.runtime.status = "stopped".to_string();
    terminal
        .draw(|frame| crate::tui::view::render(frame, &app))
        .unwrap();
    let _ = find_cell(terminal.backend().buffer(), "○ idle");

    // 2. Running state (connected)
    app.data.runtime.pid_running = true;
    app.data.runtime.status = "running".to_string();
    app.data.runtime.active_config = Some("#1 MyServer".to_string());
    terminal
        .draw(|frame| crate::tui::view::render(frame, &app))
        .unwrap();
    let _ = find_cell(terminal.backend().buffer(), "● ");
    let _ = find_cell(terminal.backend().buffer(), "#1 MyServer");

    // 3. In-flight connecting state
    app.data.runtime.pid_running = false;
    app.data.runtime.status = "starting".to_string();
    terminal
        .draw(|frame| crate::tui::view::render(frame, &app))
        .unwrap();
    let _ = find_cell(terminal.backend().buffer(), "connecting");
}

fn snapshot(buffer: &Buffer, name: &str) {
    let Ok(directory) = std::env::var("XRAT_THEME_SNAPSHOT_DIR") else {
        return;
    };
    let directory = std::path::Path::new(&directory);
    std::fs::create_dir_all(directory).unwrap();
    let cells: Vec<_> = buffer.content.iter().map(|cell| serde_json::json!({
        "text": cell.symbol(), "fg": format!("{:?}", cell.fg), "bg": format!("{:?}", cell.bg),
    })).collect();
    std::fs::write(
        directory.join(format!("{name}.json")),
        serde_json::to_vec(&serde_json::json!({
            "width": buffer.area.width, "height": buffer.area.height, "cells": cells,
        }))
        .unwrap(),
    )
    .unwrap();
}
