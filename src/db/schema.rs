use sqlx::{PgPool, SqlitePool};

static SQLITE_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
static POSTGRES_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/postgres");

pub async fn init_sqlite(pool: &SqlitePool) -> crate::db::Result<()> {
    SQLITE_MIGRATOR.run(pool).await?;
    Ok(())
}

pub async fn init_postgres(pool: &PgPool) -> crate::db::Result<()> {
    POSTGRES_MIGRATOR.run(pool).await?;
    Ok(())
}
