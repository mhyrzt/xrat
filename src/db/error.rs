#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database I/O failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("database migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("database query failed: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid runtime session status in database: {0}")]
    InvalidRuntimeSessionStatus(String),
}

pub type Result<T> = std::result::Result<T, DbError>;
