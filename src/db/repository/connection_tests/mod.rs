mod insert_ops;
mod query_ops;

pub use insert_ops::{insert, insert_run};
pub use query_ops::{get_count, get_latest_by_config, get_latest_run, list_by_config, list_by_run};
