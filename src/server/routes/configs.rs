use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::db::ConfigListFilter;
use crate::server::auth::require_api_key;
use crate::server::response::{ApiConfigDetail, PaginatedResponse, detail_from_joined};
use crate::server::{ServerError, ServerResult, ServerState};

#[derive(Debug, Deserialize)]
pub struct ConfigsQuery {
    pub key: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub enabled: Option<bool>,
    pub protocol: Option<String>,
    pub selected: Option<bool>,
}

pub async fn list_configs(
    State(state): State<ServerState>,
    Query(query): Query<ConfigsQuery>,
) -> ServerResult<Json<PaginatedResponse<ApiConfigDetail>>> {
    require_api_key(&state, query.key.as_deref())?;
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);
    if page == 0 {
        return Err(ServerError::InvalidQuery(
            "page must be greater than zero".to_string(),
        ));
    }
    if per_page == 0 || per_page > 200 {
        return Err(ServerError::InvalidQuery(
            "per_page must be between 1 and 200".to_string(),
        ));
    }

    let filter = ConfigListFilter {
        only_enabled: query.enabled.unwrap_or(false),
        only_selected: query.selected.unwrap_or(false),
        only_active: false,
        subscription_id: None,
        protocol: query.protocol.clone(),
    };

    let total = state.db.count_filtered_configs(&filter).await? as usize;
    let offset = ((page - 1) * per_page) as i64;
    let limit = per_page as i64;
    let rows = state
        .db
        .list_configs_paginated_with_latest_tests(&filter, offset, limit)
        .await?;
    let items = rows.into_iter().map(detail_from_joined).collect();

    Ok(Json(PaginatedResponse {
        total,
        page,
        per_page,
        items,
    }))
}

pub async fn get_config(
    State(state): State<ServerState>,
    Path(id): Path<i64>,
    Query(query): Query<ConfigsQuery>,
) -> ServerResult<Json<ApiConfigDetail>> {
    require_api_key(&state, query.key.as_deref())?;
    let row = state
        .db
        .get_config_with_latest_test(id)
        .await?
        .ok_or(ServerError::NotFound)?;

    Ok(Json(detail_from_joined(row)))
}
