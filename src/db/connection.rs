use std::path::Path;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::db::schema;

pub async fn connect(database_path: &Path) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    if let Some(parent) = database_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let options = SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    schema::init(&pool).await?;
    Ok(pool)
}
