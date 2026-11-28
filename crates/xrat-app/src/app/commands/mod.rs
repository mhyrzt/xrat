mod add;
mod completions;
mod connect;
mod daemon;
mod daemon_install;
#[cfg(target_os = "linux")]
pub(crate) use daemon_install::systemd_user_dir_with_env;
mod db;
mod disconnect;
pub mod gateway;
mod geoip;
mod import;
mod init;
mod lifecycle;
mod list;
mod logs;
#[cfg(any(windows, test))]
pub mod machine;
mod manpage;
pub(crate) use crate::app::terminal::output;
mod parse;
pub(crate) use crate::app::terminal::progress;
mod proxy;
mod purge;
mod resolve;
mod rotate;
mod runtime_output;
mod scan;
mod serve;
mod setup;
mod status;
pub(crate) mod test;
mod tui;
pub(crate) mod tun;
pub(crate) mod update;
pub mod upgrade;
pub mod validate;
mod version;

use crate::app::context::AppContext;
use crate::cli::Command;

mod dispatch;
pub use dispatch::run;

pub use serve::password_hash;
