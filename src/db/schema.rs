use sqlx::{PgPool, SqlitePool};

static SQLITE_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
static POSTGRES_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/postgres");

pub async fn init_sqlite(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    SQLITE_MIGRATOR.run(pool).await?;
    Ok(())
}

pub async fn init_postgres(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    POSTGRES_MIGRATOR.run(pool).await?;
    Ok(())
}
