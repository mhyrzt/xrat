mod detail;
mod filter;
mod log;
mod runtime;
mod source_detail;
mod sources_table;
mod stats;
mod table;
mod testbar;

use ratatui::Frame;

use crate::tui::app::{TuiApp, TuiPanel, TuiView};

mod render;
pub use render::render;
