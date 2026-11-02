use super::*;
use crate::tui::view::DashboardLayout;

pub fn render(frame: &mut Frame<'_>, layout: &DashboardLayout, app: &TuiApp) {
    filter::render(frame, layout.filter, app);
    testbar::render(frame, layout.testbar, app);

    let panel = app.focused_panel;
    match app.active_view {
        TuiView::Configs => {
            table::render(frame, layout.table, app, panel == TuiPanel::Table);
            detail::render(frame, layout.detail, app, panel == TuiPanel::Detail);
        }
        TuiView::Sources => {
            sources_table::render(frame, layout.table, app, panel == TuiPanel::Table);
            source_detail::render(frame, layout.detail, app, panel == TuiPanel::Detail);
        }
    }

    log::render(frame, layout.log, app, panel == TuiPanel::Log);
    runtime::render(frame, layout.runtime, app, panel == TuiPanel::Runtime);
}
