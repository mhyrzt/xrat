use crate::server::{ServerError, ServerResult, ServerState};

pub fn require_api_key(state: &ServerState, provided: Option<&str>) -> ServerResult<()> {
    let Some(expected) = state.api_key.as_deref() else {
        return Ok(());
    };
    let Some(provided) = provided else {
        return Err(ServerError::MissingApiKey);
    };
    if provided != expected {
        return Err(ServerError::InvalidApiKey);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, DatabaseConnectionConfig};

    #[tokio::test]
    async fn allows_requests_when_no_key_is_configured() {
        let state = test_state(None).await;

        require_api_key(&state, None).expect("auth should pass");
    }

    #[tokio::test]
    async fn rejects_missing_key_when_required() {
        let state = test_state(Some("secret")).await;

        assert!(matches!(
            require_api_key(&state, None),
            Err(ServerError::MissingApiKey)
        ));
    }

    #[tokio::test]
    async fn rejects_wrong_key_when_required() {
        let state = test_state(Some("secret")).await;

        assert!(matches!(
            require_api_key(&state, Some("wrong")),
            Err(ServerError::InvalidApiKey)
        ));
    }

    #[tokio::test]
    async fn allows_matching_key_when_required() {
        let state = test_state(Some("secret")).await;

        require_api_key(&state, Some("secret")).expect("auth should pass");
    }

    async fn test_state(api_key: Option<&str>) -> ServerState {
        let db = Database::connect(&DatabaseConnectionConfig::Sqlite {
            path: std::env::temp_dir().join(format!(
                "xrat-server-auth-{}.sqlite",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("time should be valid")
                    .as_nanos()
            )),
        })
        .await
        .expect("database should connect");

        ServerState {
            db,
            api_key: api_key.map(str::to_string),
            pac_enabled: true,
            pac_allowed_hosts: crate::app::config::defaults::DEFAULT_SERVER_PAC_ALLOWED_HOSTS
                .iter()
                .map(|host| host.to_string())
                .collect(),
            pac_rules: crate::server::PacRules::default(),
        }
    }
}
