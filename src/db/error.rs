#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database I/O failed")]
    Io(#[from] std::io::Error),

    #[error("database migration failed")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("database query failed")]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid runtime session status in database: {0}")]
    InvalidRuntimeSessionStatus(String),
}

pub type Result<T> = std::result::Result<T, DbError>;
