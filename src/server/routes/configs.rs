use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::db::{ConfigListFilter, RefMatch};
use crate::server::auth::require_api_key;
use crate::server::response::{ApiConfigDetail, PaginatedResponse, detail_from_joined};
use crate::server::{ServerError, ServerResult, ServerState};
use crate::support::refs::is_ref_prefix;

#[derive(Debug, Deserialize)]
pub struct ConfigsQuery {
    pub key: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub enabled: Option<bool>,
    pub protocol: Option<String>,
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
        only_active: false,
        only_deleted: false,
        include_deleted: false,
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
    Path(id): Path<String>,
    Query(query): Query<ConfigsQuery>,
) -> ServerResult<Json<ApiConfigDetail>> {
    require_api_key(&state, query.key.as_deref())?;
    let id = resolve_config_identifier(&state, &id).await?;
    let row = state
        .db
        .get_config_with_latest_test(id)
        .await?
        .ok_or(ServerError::NotFound)?;

    Ok(Json(detail_from_joined(row)))
}

async fn resolve_config_identifier(state: &ServerState, raw: &str) -> ServerResult<i64> {
    if let Some(id) = existing_numeric_config_id(state, raw).await? {
        return Ok(id);
    }

    if is_ref_prefix(raw) {
        return match state.db.resolve_config_ref_prefix(raw).await? {
            RefMatch::Unique(id) => Ok(id),
            RefMatch::Ambiguous => Err(ServerError::InvalidQuery(format!(
                "config ref prefix '{raw}' is ambiguous; provide more characters"
            ))),
            RefMatch::None => Err(ServerError::NotFound),
        };
    }

    Err(ServerError::NotFound)
}

async fn existing_numeric_config_id(state: &ServerState, raw: &str) -> ServerResult<Option<i64>> {
    let Ok(id) = raw.parse::<i64>() else {
        return Ok(None);
    };
    Ok(state.db.get_config_by_id(id).await?.is_some().then_some(id))
}
