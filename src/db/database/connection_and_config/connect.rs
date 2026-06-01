use super::super::Database;
use super::super::types::*;

impl Database {
    pub async fn connect(config: &DatabaseConnectionConfig) -> crate::db::Result<Self> {
        let pool = connection::connect(config).await?;
        Ok(Self { pool })
    }

    #[cfg(test)]
    pub(crate) async fn connect_sqlite(database_path: &std::path::Path) -> crate::db::Result<Self> {
        Self::connect(&DatabaseConnectionConfig::Sqlite {
            path: database_path.to_path_buf(),
        })
        .await
    }

    #[cfg(test)]
    pub(crate) async fn connect_postgres_url(url: String) -> crate::db::Result<Self> {
        Self::connect(&DatabaseConnectionConfig::Postgres {
            url,
            max_connections: 5,
            min_connections: 0,
            connect_timeout: Duration::from_secs(10),
        })
        .await
    }

    #[cfg(test)]
    pub(crate) async fn clear_for_test(&self) -> crate::db::Result<()> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query("DELETE FROM runtime_sessions")
                    .execute(pool)
                    .await?;
                sqlx::query("DELETE FROM connection_tests")
                    .execute(pool)
                    .await?;
                sqlx::query("DELETE FROM connection_test_runs")
                    .execute(pool)
                    .await?;
                sqlx::query("DELETE FROM cf_scan_results")
                    .execute(pool)
                    .await?;
                sqlx::query("DELETE FROM configs").execute(pool).await?;
                sqlx::query("DELETE FROM subscriptions")
                    .execute(pool)
                    .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "TRUNCATE runtime_sessions, connection_tests, connection_test_runs, cf_scan_results, configs, subscriptions RESTART IDENTITY CASCADE",
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
}
