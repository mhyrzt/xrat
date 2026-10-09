use super::*;

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(area);

    render_body(frame, shell[0], app);
    chrome::render_key_bar(frame, shell[1], app);

    if app.show_help {
        modals::render_help(frame, area);
    }

    if app.import_modal.is_some() {
        modals::render_import_modal(frame, area, app);
    }

    if app.settings_modal.is_some() {
        modals::render_settings_modal(frame, area, app);
    }

    if app.split_modal.is_some() {
        modals::render_split_modal(frame, area, app);
    }

    if app.rename_modal.is_some() {
        modals::render_rename_modal(frame, area, app);
    }

    if app.qr_modal.is_some() {
        modals::render_qr_modal(frame, area, app);
    }
}

pub(super) fn render_body(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    configs::render(frame, area, app);
}
