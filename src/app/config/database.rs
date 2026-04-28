use std::path::PathBuf;

use serde::Deserialize;

use super::SecretString;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DatabaseSettings {
    pub backend: DatabaseBackend,
    pub sqlite: SqliteDatabaseSettings,
    pub postgres: PostgresDatabaseSettings,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            backend: DatabaseBackend::Sqlite,
            sqlite: SqliteDatabaseSettings::default(),
            postgres: PostgresDatabaseSettings::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    #[default]
    Sqlite,
    Postgres,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SqliteDatabaseSettings {
    pub path: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PostgresDatabaseSettings {
    pub user: SecretString,
    pub password: SecretString,
    pub host: String,
    pub port: u16,
    pub db_name: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
}

impl Default for PostgresDatabaseSettings {
    fn default() -> Self {
        Self {
            user: SecretString::Literal(String::new()),
            password: SecretString::Literal(String::new()),
            host: "localhost".to_string(),
            port: 5432,
            db_name: String::new(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout_secs: 10,
        }
    }
}
