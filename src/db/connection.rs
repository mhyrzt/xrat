use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{PgPool, SqlitePool};

use crate::db::schema;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseConnectionConfig {
    Sqlite {
        path: PathBuf,
    },
    Postgres {
        url: String,
        max_connections: u32,
        min_connections: u32,
        connect_timeout: Duration,
    },
}

impl DatabaseConnectionConfig {
    pub fn label(&self) -> String {
        match self {
            Self::Sqlite { path } => path.display().to_string(),
            Self::Postgres { url, .. } => redact_postgres_url(url),
        }
    }
}

#[derive(Clone)]
pub enum DbPool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
}

pub async fn connect(config: &DatabaseConnectionConfig) -> crate::db::Result<DbPool> {
    match config {
        DatabaseConnectionConfig::Sqlite { path } => connect_sqlite(path).await,
        DatabaseConnectionConfig::Postgres {
            url,
            max_connections,
            min_connections,
            connect_timeout,
        } => connect_postgres(url, *max_connections, *min_connections, *connect_timeout).await,
    }
}

async fn connect_sqlite(database_path: &Path) -> crate::db::Result<DbPool> {
    if let Some(parent) = database_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true)
        .pragma("foreign_keys", "ON");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    schema::init_sqlite(&pool).await?;
    Ok(DbPool::Sqlite(pool))
}

async fn connect_postgres(
    url: &str,
    max_connections: u32,
    min_connections: u32,
    connect_timeout: Duration,
) -> crate::db::Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(connect_timeout)
        .connect(url)
        .await?;

    schema::init_postgres(&pool).await?;
    Ok(DbPool::Postgres(pool))
}

fn redact_postgres_url(url: &str) -> String {
    let Some((prefix, rest)) = url.split_once("://") else {
        return "postgres://<redacted>".to_string();
    };
    let Some((userinfo, host)) = rest.rsplit_once('@') else {
        return format!("{prefix}://{rest}");
    };
    let Some((user, _password)) = userinfo.split_once(':') else {
        return format!("{prefix}://{userinfo}@{host}");
    };

    format!("{prefix}://{user}:<redacted>@{host}")
}

#[cfg(test)]
mod tests {
    use super::{DatabaseConnectionConfig, redact_postgres_url};

    #[test]
    fn redacts_postgres_password_in_label() {
        assert_eq!(
            redact_postgres_url("postgres://user:secret@localhost:5432/xrat"),
            "postgres://user:<redacted>@localhost:5432/xrat"
        );
    }

    #[test]
    fn labels_sqlite_database_path() {
        let config = DatabaseConnectionConfig::Sqlite {
            path: "/tmp/xrat/db.sqlite".into(),
        };

        assert_eq!(config.label(), "/tmp/xrat/db.sqlite");
    }
}
