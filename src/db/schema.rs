use sqlx::SqlitePool;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub async fn init(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    MIGRATOR.run(pool).await?;
    Ok(())
}
