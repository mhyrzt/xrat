use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::context::AppContext;
use crate::tui::app::{TuiApp, TuiConfigCommand};
use crate::tui::data::TuiData;

pub async fn run(context: &AppContext) -> crate::app::Result<()> {
    let mut terminal = TerminalSession::enter()?;
    let data = TuiData::load(context, false).await?;
    let mut app = TuiApp::with_data(data);

    loop {
        terminal.draw(|frame| crate::tui::view::render(frame, &app))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let action = crate::tui::keymap::action_for_key(
                    key,
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
                    reload_data(context, &mut app).await;
                }
                if let Some(command) = confirmed_command.or(direct_command) {
                    run_config_command(context, &mut app, command).await;
                }
            }
        }
    }

    Ok(())
}

async fn run_config_command(context: &AppContext, app: &mut TuiApp, command: TuiConfigCommand) {
    let result = match command {
        TuiConfigCommand::Select(id) => context
            .db
            .set_selected_config(id)
            .await
            .map(|_| format!("selected config {id}")),
        TuiConfigCommand::Enable(id) => context
            .db
            .set_config_enabled(id, true)
            .await
            .map(|_| format!("enabled config {id}")),
        TuiConfigCommand::Disable(id) => context
            .db
            .set_config_enabled(id, false)
            .await
            .map(|_| format!("disabled config {id}")),
        TuiConfigCommand::Restore(id) => context
            .db
            .restore_config(id)
            .await
            .map(|_| format!("restored config {id}")),
        TuiConfigCommand::SoftDelete(id) => context
            .db
            .delete_config(id)
            .await
            .map(|_| format!("soft deleted config {id}")),
        TuiConfigCommand::Purge(id) => context
            .db
            .hard_delete_config(id)
            .await
            .map(|_| format!("purged config {id}")),
    };

    match result {
        Ok(message) => {
            reload_data(context, app).await;
            app.set_status(message);
        }
        Err(error) => app.set_status(format!("operation failed: {error}")),
    }
}

async fn reload_data(context: &AppContext, app: &mut TuiApp) {
    match TuiData::load(context, app.config_list.include_deleted).await {
        Ok(data) => app.reload_data(data),
        Err(error) => app.set_status(format!("reload failed: {error}")),
    }
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    fn draw<F>(&mut self, render: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame<'_>),
    {
        self.terminal.draw(render).map(|_| ())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
