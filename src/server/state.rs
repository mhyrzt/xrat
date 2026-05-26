use crate::app::config::ServerSettings;
use crate::db::Database;

#[derive(Clone)]
pub struct ServerState {
    pub db: Database,
    pub api_key: Option<String>,
}

impl ServerState {
    pub fn from_settings(
        db: Database,
        settings: &ServerSettings,
    ) -> crate::app::Result<ServerState> {
        let api_key = settings
            .key
            .as_ref()
            .map(|secret| secret.resolve())
            .transpose()?;
        Ok(Self { db, api_key })
    }
}
