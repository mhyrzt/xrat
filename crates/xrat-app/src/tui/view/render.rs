use super::*;

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    crate::tui::theme::clear(frame, area);
    let layout = DashboardLayout::new(area);

    render_body(frame, &layout, app);
    chrome::render_key_bar(frame, layout.keybar, app);

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

pub(super) fn render_body(frame: &mut Frame<'_>, layout: &DashboardLayout, app: &TuiApp) {
    configs::render(frame, layout, app);
}
