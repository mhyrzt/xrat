use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::db::ConfigListFilter;
use crate::server::auth::require_api_key;
use crate::server::response::{ApiConfigSummary, summary_from_joined};
use crate::server::{ServerError, ServerResult, ServerState};

#[derive(Debug, Deserialize)]
pub struct JsonQuery {
    pub key: Option<String>,
    pub top: Option<u64>,
    pub enabled: Option<bool>,
    pub protocol: Option<String>,
}

pub async fn json(
    State(state): State<ServerState>,
    Query(query): Query<JsonQuery>,
) -> ServerResult<Json<Vec<ApiConfigSummary>>> {
    require_api_key(&state, query.key.as_deref())?;
    let rows = list_api_configs(&state, &query).await?;
    Ok(Json(rows.iter().map(summary_from_joined).collect()))
}

pub(crate) async fn list_api_configs(
    state: &ServerState,
    query: &JsonQuery,
) -> ServerResult<Vec<crate::db::ConfigWithLatestTest>> {
    let filter = ConfigListFilter {
        only_enabled: query.enabled.unwrap_or(true),
        only_active: false,
        only_deleted: false,
        include_deleted: false,
        subscription_id: None,
        protocol: query.protocol.clone(),
    };

    if let Some(top) = query.top {
        let limit = validate_top(top)?;
        return Ok(state
            .db
            .list_top_configs_by_real_delay(limit as i64, &filter)
            .await?);
    }

    Ok(state.db.list_configs_with_latest_tests(&filter).await?)
}

fn validate_top(top: u64) -> ServerResult<usize> {
    if top == 0 || top > 200 {
        return Err(ServerError::InvalidQuery(
            "top must be between 1 and 200".to_string(),
        ));
    }
    Ok(top as usize)
}
