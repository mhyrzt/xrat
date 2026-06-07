use crate::app::config::{RoutingSettings, ServerSettings};
use crate::db::Database;
use crate::server::routes::pac::PacRules;

#[derive(Clone)]
pub struct ServerState {
    pub db: Database,
    pub api_key: Option<String>,
    pub pac_enabled: bool,
    pub pac_allowed_hosts: Vec<String>,
    pub pac_rules: PacRules,
}

impl ServerState {
    pub fn from_settings(
        db: Database,
        settings: &ServerSettings,
        routing: &RoutingSettings,
    ) -> crate::app::Result<ServerState> {
        let api_key = settings
            .key
            .as_ref()
            .map(|secret| secret.resolve())
            .transpose()?;
        Ok(Self {
            db,
            api_key,
            pac_enabled: settings.pac_enabled,
            pac_allowed_hosts: settings.pac_allowed_hosts.clone(),
            pac_rules: PacRules::from_routing(routing),
        })
    }
}
