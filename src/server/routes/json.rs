use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::db::{ConfigListFilter, ConfigRecord, ConnectionTestRecord};
use crate::server::auth::require_api_key;
use crate::server::response::{ApiConfigSummary, summary_response};
use crate::server::{ServerError, ServerResult, ServerState};

#[derive(Debug, Deserialize)]
pub struct JsonQuery {
    pub key: Option<String>,
    pub top: Option<u64>,
    pub enabled: Option<bool>,
    pub protocol: Option<String>,
    pub selected: Option<bool>,
}

pub async fn json(
    State(state): State<ServerState>,
    Query(query): Query<JsonQuery>,
) -> ServerResult<Json<Vec<ApiConfigSummary>>> {
    require_api_key(&state, query.key.as_deref())?;
    let rows = list_configs_with_latest_tests(&state, &query).await?;
    Ok(Json(
        rows.iter()
            .map(|(config, latest_test)| summary_response(config, latest_test.as_ref()))
            .collect(),
    ))
}

pub(crate) async fn list_configs_with_latest_tests(
    state: &ServerState,
    query: &JsonQuery,
) -> ServerResult<Vec<(ConfigRecord, Option<ConnectionTestRecord>)>> {
    let top = validate_top(query.top)?;
    let filter = ConfigListFilter {
        only_enabled: query.enabled.unwrap_or(true),
        only_selected: query.selected.unwrap_or(false),
        only_active: false,
        subscription_id: None,
    };
    let mut configs = state.db.list_configs(&filter).await?;
    if let Some(protocol) = query.protocol.as_deref() {
        configs.retain(|config| config.protocol == protocol);
    }

    let mut rows = Vec::with_capacity(configs.len());
    for config in configs {
        let latest_test = state.db.get_latest_connection_test(config.id).await?;
        rows.push((config, latest_test));
    }

    if let Some(top) = top {
        rows.retain(|(_, latest_test)| {
            latest_test
                .as_ref()
                .and_then(|test| test.real_delay_ms)
                .is_some()
        });
        rows.sort_by_key(|(_, latest_test)| {
            latest_test
                .as_ref()
                .and_then(|test| test.real_delay_ms)
                .unwrap_or(i64::MAX)
        });
        rows.truncate(top);
    }

    Ok(rows)
}

fn validate_top(top: Option<u64>) -> ServerResult<Option<usize>> {
    let Some(top) = top else {
        return Ok(None);
    };
    if top == 0 || top > 200 {
        return Err(ServerError::InvalidQuery(
            "top must be between 1 and 200".to_string(),
        ));
    }
    Ok(Some(top as usize))
}
