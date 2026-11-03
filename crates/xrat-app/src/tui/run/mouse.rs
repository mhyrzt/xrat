use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};

use super::terminal::{TuiMouseButton, TuiMouseEvent, TuiMouseEventKind};
use crate::tui::app::{SettingsPane, TuiAction, TuiApp, TuiLogTab, TuiPanel, TuiView, scroll};
use crate::tui::view::DashboardLayout;
use crate::tui::view::release_notes::ReleaseNotesModal;

pub(super) fn handle_mouse_event(
    app: &mut TuiApp,
    event: &TuiMouseEvent,
    terminal_size: (u16, u16),
    release_notes: &mut Option<ReleaseNotesModal>,
) -> bool {
    let screen = Rect::new(0, 0, terminal_size.0, terminal_size.1);
    let pos = Position::new(event.column, event.row);

    // 1. Release notes modal takes top priority if open.
    if let Some(modal) = release_notes.as_mut() {
        match event.kind {
            TuiMouseEventKind::ScrollUp => {
                modal.handle_scroll(-2);
                return true;
            }
            TuiMouseEventKind::ScrollDown => {
                modal.handle_scroll(2);
                return true;
            }
            TuiMouseEventKind::Down(TuiMouseButton::Left) => {
                let width = screen.width.saturating_sub(4).min(100);
                let height = screen.height.saturating_sub(2);
                let modal_area = Rect::new(
                    screen.x + (screen.width - width) / 2,
                    screen.y + (screen.height - height) / 2,
                    width,
                    height,
                );
                if !modal_area.contains(pos) {
                    *release_notes = None;
                    app.needs_full_clear = true;
                    return true;
                }
                return false;
            }
            _ => return false,
        }
    }

    // 2. Help modal
    if app.show_help {
        if matches!(event.kind, TuiMouseEventKind::Down(TuiMouseButton::Left)) {
            app.show_help = false;
            return true;
        }
        return false;
    }

    // 3. Settings modal
    if let Some(modal) = &mut app.settings_modal {
        let width = screen.width.saturating_sub(4).clamp(50, 110);
        let height = screen.height.saturating_sub(2).clamp(20, 40);
        let modal_area = Rect::new(
            screen.x + (screen.width.saturating_sub(width)) / 2,
            screen.y + (screen.height.saturating_sub(height)) / 2,
            width,
            height,
        );

        match event.kind {
            TuiMouseEventKind::Down(TuiMouseButton::Left) => {
                if !modal_area.contains(pos) {
                    app.apply(TuiAction::Back);
                    return true;
                }
                let inner = Rect::new(
                    modal_area.x + 2,
                    modal_area.y + 1,
                    modal_area.width.saturating_sub(4),
                    modal_area.height.saturating_sub(2),
                );
                let compact = modal_area.width < 80;
                let rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(7),
                        Constraint::Length(if compact { 11 } else { 10 }),
                        Constraint::Length(
                            if modal.error.is_some()
                                || modal.notice.is_some()
                                || modal.discard_confirm
                            {
                                2
                            } else if modal.selected_setting_index().is_none() {
                                1
                            } else {
                                0
                            },
                        ),
                    ])
                    .split(inner);

                let columns = if compact {
                    match modal.pane {
                        SettingsPane::Sections => [rows[1], Rect::default()],
                        SettingsPane::Fields => [Rect::default(), rows[1]],
                    }
                } else {
                    let cols = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                        .split(rows[1]);
                    [cols[0], cols[1]]
                };

                if columns[0].contains(pos) {
                    modal.pane = SettingsPane::Sections;
                    let sections = modal.sections();
                    let section_rows = columns[0].height.saturating_sub(2) as usize;
                    let section_start = modal
                        .section_index
                        .saturating_sub(section_rows.saturating_sub(1));
                    let clicked = (event.row.saturating_sub(columns[0].y + 1)) as usize;
                    let target = section_start + clicked;
                    if target < sections.len() {
                        modal.section_index = target;
                        modal.field_index = 0;
                    }
                    return true;
                } else if columns[1].contains(pos) {
                    modal.pane = SettingsPane::Fields;
                    let available_rows = columns[1].height.saturating_sub(2) as usize;
                    let layout =
                        crate::tui::view::modals::settings_field_layout(modal, available_rows);
                    let clicked = (event.row.saturating_sub(columns[1].y + 1)) as usize;
                    let row_idx = layout.start + clicked;
                    if let Some(&Some(target)) = layout.rows.get(row_idx) {
                        if modal.field_index == target {
                            app.apply(TuiAction::SettingsSubmit);
                        } else {
                            modal.field_index = target;
                        }
                    }
                    return true;
                }
                return false;
            }
            TuiMouseEventKind::ScrollUp => {
                app.apply(TuiAction::SettingsMove(-1));
                return true;
            }
            TuiMouseEventKind::ScrollDown => {
                app.apply(TuiAction::SettingsMove(1));
                return true;
            }
            _ => return false,
        }
    }

    // 4. Other modals: close on click outside
    if app.import_modal.is_some()
        || app.rename_modal.is_some()
        || app.qr_modal.is_some()
        || app.split_modal.is_some()
    {
        if matches!(event.kind, TuiMouseEventKind::Down(TuiMouseButton::Left)) {
            let width = screen.width.saturating_sub(6).min(70);
            let height = 8u16;
            let modal_area = Rect::new(
                screen.x + (screen.width.saturating_sub(width)) / 2,
                screen.y + (screen.height.saturating_sub(height)) / 2,
                width,
                height,
            );
            if !modal_area.contains(pos) {
                app.apply(TuiAction::Back);
                return true;
            }
        }
        return false;
    }

    // 5. Confirm or bulk operation in key bar
    if app.confirm.is_some() || app.pending_bulk.is_some() {
        if matches!(event.kind, TuiMouseEventKind::Down(TuiMouseButton::Left)) {
            let layout = DashboardLayout::new(screen);
            if layout.keybar.contains(pos) {
                // Clicking in key bar on confirmation
                return false;
            }
        }
        return false;
    }

    // 6. Main Dashboard
    let layout = DashboardLayout::new(screen);

    match event.kind {
        TuiMouseEventKind::ScrollUp => {
            match layout.panel_at(event.column, event.row) {
                Some(TuiPanel::Table) => match app.active_view {
                    TuiView::Configs => app.move_config_focus(-1),
                    TuiView::Sources => app.move_source_focus(-1),
                },
                Some(TuiPanel::Detail) => scroll(&app.panel_scroll.detail, -2),
                Some(TuiPanel::Log) => scroll(&app.panel_scroll.log, -2),
                Some(TuiPanel::Runtime) => scroll(&app.panel_scroll.runtime, -2),
                None => {
                    if app.focused_panel == TuiPanel::Table {
                        app.move_focus(-1);
                    } else {
                        app.move_focus(-2);
                    }
                }
            }
            true
        }
        TuiMouseEventKind::ScrollDown => {
            match layout.panel_at(event.column, event.row) {
                Some(TuiPanel::Table) => match app.active_view {
                    TuiView::Configs => app.move_config_focus(1),
                    TuiView::Sources => app.move_source_focus(1),
                },
                Some(TuiPanel::Detail) => scroll(&app.panel_scroll.detail, 2),
                Some(TuiPanel::Log) => scroll(&app.panel_scroll.log, 2),
                Some(TuiPanel::Runtime) => scroll(&app.panel_scroll.runtime, 2),
                None => {
                    if app.focused_panel == TuiPanel::Table {
                        app.move_focus(1);
                    } else {
                        app.move_focus(2);
                    }
                }
            }
            true
        }
        TuiMouseEventKind::Down(TuiMouseButton::Left) => {
            if layout.table.contains(pos) {
                app.focused_panel = TuiPanel::Table;

                // Check title bar row for tab switching
                if event.row == layout.table.y {
                    let rel_x = event.column.saturating_sub(layout.table.x);
                    if (3..=13).contains(&rel_x) {
                        app.active_view = TuiView::Configs;
                        app.panel_scroll.detail.set(0);
                    } else if (14..=30).contains(&rel_x) {
                        app.active_view = TuiView::Sources;
                        app.panel_scroll.detail.set(0);
                    }
                    return true;
                }

                // Check table data rows
                let header_offset = 2u16;
                if event.row >= layout.table.y + header_offset
                    && event.row < layout.table.y + layout.table.height.saturating_sub(1)
                {
                    let row_in_view = (event.row - (layout.table.y + header_offset)) as usize;
                    match app.active_view {
                        TuiView::Configs => {
                            let visible_len = app.visible_configs().len();
                            let target = app.config_list.table_offset.get() + row_in_view;
                            if target < visible_len {
                                app.config_list.focused = target;
                                app.panel_scroll.detail.set(0);
                            }
                        }
                        TuiView::Sources => {
                            let total_rows = 2 + app.data.sources.len();
                            let target = app.source_list.table_offset.get() + row_in_view;
                            if target < total_rows {
                                app.source_list.focused = target;
                                app.sync_source_filter();
                                app.panel_scroll.detail.set(0);
                            }
                        }
                    }
                }
                return true;
            }

            if layout.detail.contains(pos) {
                app.focused_panel = TuiPanel::Detail;
                return true;
            }

            if layout.log.contains(pos) {
                app.focused_panel = TuiPanel::Log;

                // Check title bar row for log tab switching
                if event.row == layout.log.y {
                    let rel_x = event.column.saturating_sub(layout.log.x);
                    if (4..=13).contains(&rel_x) {
                        app.active_log_tab = TuiLogTab::XratEvents;
                    } else if (14..=22).contains(&rel_x) {
                        app.active_log_tab = TuiLogTab::ProxyEngine;
                    } else if (23..=32).contains(&rel_x) {
                        app.active_log_tab = TuiLogTab::Stats;
                    } else if (33..=42).contains(&rel_x) {
                        app.active_log_tab = TuiLogTab::Api;
                    }
                }
                return true;
            }

            if layout.runtime.contains(pos) {
                app.focused_panel = TuiPanel::Runtime;
                return true;
            }

            if layout.filter.contains(pos) {
                let rel_x = event.column.saturating_sub(layout.filter.x);
                if rel_x <= 18 {
                    if app.config_list.editing_search {
                        app.apply(TuiAction::ConfirmSearch);
                    } else {
                        app.apply(TuiAction::BeginSearch);
                    }
                } else if rel_x <= 36 {
                    app.apply(TuiAction::CycleSort);
                } else {
                    app.apply(TuiAction::CycleFilter);
                }
                return true;
            }

            if layout.keybar.contains(pos) {
                let rel_x = event.column.saturating_sub(layout.keybar.x);
                if rel_x <= 10 {
                    app.show_help = true;
                } else if (12..=25).contains(&rel_x) {
                    app.apply(TuiAction::OpenSettingsModal);
                } else if rel_x >= layout.keybar.width.saturating_sub(8) {
                    app.should_quit = true;
                }
                return true;
            }

            false
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
