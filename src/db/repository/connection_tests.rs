use sqlx::SqlitePool;

pub async fn get_count(pool: &SqlitePool) -> Result<i64, Box<dyn std::error::Error>> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM connection_tests")
            .fetch_one(pool)
            .await?,
    )
}
