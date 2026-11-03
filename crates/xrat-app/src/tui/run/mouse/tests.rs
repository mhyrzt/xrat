use super::*;
use crate::tui::data::{TuiConfigRow, TuiData};

fn test_app_with_configs(count: usize) -> TuiApp {
    let mut data = TuiData::default();
    for i in 1..=count {
        data.configs.push(TuiConfigRow {
            id: xrat_model::ConfigId(i as i64),
            name: format!("server-{i}"),
            is_enabled: true,
            is_deleted: false,
            ..Default::default()
        });
    }
    data.total_configs = count;
    data.enabled_configs = count;
    TuiApp::with_data(data)
}

#[test]
fn dashboard_layout_panels_and_hit_testing() {
    let layout = DashboardLayout::new(Rect::new(0, 0, 100, 40));
    assert_eq!(
        layout.panel_at(layout.table.x + 2, layout.table.y + 2),
        Some(TuiPanel::Table)
    );
    assert_eq!(
        layout.panel_at(layout.detail.x + 2, layout.detail.y + 2),
        Some(TuiPanel::Detail)
    );
    assert_eq!(
        layout.panel_at(layout.log.x + 2, layout.log.y + 2),
        Some(TuiPanel::Log)
    );
    assert_eq!(
        layout.panel_at(layout.runtime.x + 2, layout.runtime.y + 2),
        Some(TuiPanel::Runtime)
    );
}

#[test]
fn mouse_scroll_on_table_moves_config_focus_by_one() {
    let mut app = test_app_with_configs(10);
    assert_eq!(app.config_list.focused, 0);

    let layout = DashboardLayout::new(Rect::new(0, 0, 100, 40));
    let event_down = TuiMouseEvent {
        kind: TuiMouseEventKind::ScrollDown,
        column: layout.table.x + 5,
        row: layout.table.y + 5,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    let redraw = handle_mouse_event(&mut app, &event_down, (100, 40), &mut None);
    assert!(redraw);
    assert_eq!(app.config_list.focused, 1);

    let event_up = TuiMouseEvent {
        kind: TuiMouseEventKind::ScrollUp,
        column: layout.table.x + 5,
        row: layout.table.y + 5,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    let redraw = handle_mouse_event(&mut app, &event_up, (100, 40), &mut None);
    assert!(redraw);
    assert_eq!(app.config_list.focused, 0);
}

#[test]
fn mouse_scroll_on_text_panels_scrolls_by_two_lines() {
    let mut app = test_app_with_configs(5);
    app.panel_scroll.detail.set(10);
    app.panel_scroll.log.set(10);
    app.panel_scroll.runtime.set(10);

    let layout = DashboardLayout::new(Rect::new(0, 0, 100, 40));

    // Scroll Detail
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::ScrollUp,
            column: layout.detail.x + 2,
            row: layout.detail.y + 2,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.panel_scroll.detail.get(), 8);

    // Scroll Log
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::ScrollDown,
            column: layout.log.x + 2,
            row: layout.log.y + 2,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.panel_scroll.log.get(), 12);

    // Scroll Runtime
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::ScrollDown,
            column: layout.runtime.x + 2,
            row: layout.runtime.y + 2,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.panel_scroll.runtime.get(), 12);
}

#[test]
fn clicking_on_panels_switches_active_focus() {
    let mut app = test_app_with_configs(5);
    assert_eq!(app.focused_panel, TuiPanel::Table);

    let layout = DashboardLayout::new(Rect::new(0, 0, 100, 40));

    // Click on Detail
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.detail.x + 5,
            row: layout.detail.y + 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.focused_panel, TuiPanel::Detail);

    // Click on Log
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.log.x + 5,
            row: layout.log.y + 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.focused_panel, TuiPanel::Log);

    // Click on Runtime
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.runtime.x + 5,
            row: layout.runtime.y + 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.focused_panel, TuiPanel::Runtime);

    // Click on Table
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.table.x + 5,
            row: layout.table.y + 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.focused_panel, TuiPanel::Table);
}

#[test]
fn clicking_on_table_row_selects_config_with_offset() {
    let mut app = test_app_with_configs(30);
    let layout = DashboardLayout::new(Rect::new(0, 0, 100, 40));

    // Case 1: offset is 0, click on row 3 (data rows start at y + 2, so row 3 is at y + 2 + 3 = y + 5)
    app.config_list.table_offset.set(0);
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.table.x + 10,
            row: layout.table.y + 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.config_list.focused, 3);

    // Case 2: offset is 15 (table has scrolled down), click on row 2
    app.config_list.table_offset.set(15);
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.table.x + 10,
            row: layout.table.y + 4, // 4 - 2 = 2
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.config_list.focused, 17);
}

#[test]
fn clicking_on_card_title_tabs_switches_tabs() {
    let mut app = test_app_with_configs(5);
    let layout = DashboardLayout::new(Rect::new(0, 0, 100, 40));

    // Table title: Subscriptions tab
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.table.x + 18,
            row: layout.table.y,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.active_view, TuiView::Sources);

    // Table title: Configs tab
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.table.x + 5,
            row: layout.table.y,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.active_view, TuiView::Configs);

    // Log title: Engine tab
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.log.x + 16,
            row: layout.log.y,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.active_log_tab, TuiLogTab::ProxyEngine);

    // Log title: API tab
    handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: layout.log.x + 35,
            row: layout.log.y,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (100, 40),
        &mut None,
    );
    assert_eq!(app.active_log_tab, TuiLogTab::Api);
}

#[test]
fn clicking_settings_modal_fields_respects_group_headers_and_layout() {
    let _guard = crate::tui::theme::TEST_THEME_LOCK.lock().unwrap();
    let mut app = test_app_with_configs(1);
    let root = tempfile::tempdir().expect("temp directory");
    let path = root.path().join("config.toml");
    std::fs::write(&path, "").expect("write");
    let session = crate::app::config::ConfigEditSession::open(&path).expect("session");
    let mut modal = crate::tui::app::SettingsModalState::new(session);
    if let Some(pos) = modal.sections().iter().position(|s| s == "runtime.tun") {
        modal.section_index = pos;
    }
    app.settings_modal = Some(modal);

    let screen_w: u16 = 100;
    let screen_h: u16 = 40;
    let width = screen_w.saturating_sub(4).clamp(50, 110);
    let height = screen_h.saturating_sub(2).clamp(20, 40);
    let modal_area = Rect::new(
        (screen_w - width) / 2,
        (screen_h - height) / 2,
        width,
        height,
    );
    let inner = Rect::new(
        modal_area.x + 2,
        modal_area.y + 1,
        modal_area.width.saturating_sub(4),
        modal_area.height.saturating_sub(2),
    );
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(10),
            Constraint::Length(0),
        ])
        .split(inner);
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(rows[1]);

    let modal_ref = app.settings_modal.as_ref().unwrap();
    let available_rows = cols[1].height.saturating_sub(2) as usize;
    let layout = crate::tui::view::modals::settings_field_layout(modal_ref, available_rows);

    let first_setting_visual_row = layout
        .rows
        .iter()
        .position(|row| row.is_some())
        .expect("should have at least one setting");
    let target_idx = layout.rows[first_setting_visual_row].unwrap();

    // Click on that row in columns[1]
    let click_y = cols[1].y + 1 + first_setting_visual_row as u16;
    let redraw = handle_mouse_event(
        &mut app,
        &TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: cols[1].x + 5,
            row: click_y,
            modifiers: crossterm::event::KeyModifiers::empty(),
        },
        (screen_w, screen_h),
        &mut None,
    );
    assert!(redraw);
    assert_eq!(app.settings_modal.as_ref().unwrap().field_index, target_idx);
}

#[test]
fn clicking_inside_split_modal_preserves_pending_changes() {
    let mut app = test_app_with_configs(1);
    let mut modal = crate::tui::app::SplitModalState::from_tun(&Default::default());
    modal.tun_enabled = !modal.tun_enabled;
    app.split_modal = Some(modal);
    let screen = Rect::new(0, 0, 120, 40);
    let area = crate::tui::view::modals::split_modal_area(screen);
    let click = TuiMouseEvent {
        kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
        column: area.x + 1,
        row: area.y + 1,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };
    handle_mouse_event(&mut app, &click, (120, 40), &mut None);
    assert!(app.split_modal.as_ref().unwrap().is_dirty());
    let outside = TuiMouseEvent {
        column: 0,
        row: 0,
        ..click
    };
    handle_mouse_event(&mut app, &outside, (120, 40), &mut None);
    assert!(app.split_modal.is_none());
}
