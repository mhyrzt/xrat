pub(crate) mod chrome;
mod configs;
pub(crate) mod modals;
mod shared;

use ratatui::Frame;

use crate::tui::app::TuiApp;

pub mod layout;
pub use layout::DashboardLayout;
mod render;
pub use render::render;

pub(crate) mod release_notes;
