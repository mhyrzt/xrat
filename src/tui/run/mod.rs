mod tasks;
mod terminal;

use std::time::Duration;

use crossterm::event::{self, Event};
use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::app::TuiApp;
use crate::tui::data::TuiData;
use crate::tui::task::TuiTaskEvent;

use terminal::TerminalSession;

pub async fn run(context: &AppContext) -> crate::app::Result<()> {
    let mut terminal = TerminalSession::enter()?;
    let data = TuiData::load(context, false).await?;
    let mut app = TuiApp::with_data(data);
    let (task_tx, mut task_rx) = mpsc::unbounded_channel();

    loop {
        drain_task_events(&mut app, &mut task_rx);
        terminal.draw(|frame| crate::tui::view::render(frame, &app))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let action = crate::tui::keymap::action_for_key(
                    key,
                    app.active_view,
                    app.config_list.editing_search,
                    app.confirm.is_some(),
                );
                let confirmed_command = if matches!(action, crate::tui::app::TuiAction::Confirm) {
                    app.pending_confirm_command()
                } else {
                    None
                };
                let direct_command = app.config_command_for_action(action);
                app.apply(action);
                if matches!(action, crate::tui::app::TuiAction::ToggleDeletedFilter) {
                    tasks::spawn_reload_data(
                        context.clone(),
                        app.config_list.include_deleted,
                        &task_tx,
                    );
                }
                if matches!(action, crate::tui::app::TuiAction::StartTestBatch) {
                    tasks::spawn_test_batch(context.clone(), &mut app, &task_tx);
                }
                if let Some(command) = confirmed_command.or(direct_command) {
                    tasks::run_config_command(context, &mut app, command).await;
                }
            }
        }
    }

    Ok(())
}

fn drain_task_events(app: &mut TuiApp, task_rx: &mut mpsc::UnboundedReceiver<TuiTaskEvent>) {
    while let Ok(event) = task_rx.try_recv() {
        app.apply_task_event(event);
    }
}
