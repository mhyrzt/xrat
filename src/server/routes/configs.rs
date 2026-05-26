use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::db::ConfigListFilter;
use crate::server::auth::require_api_key;
use crate::server::response::{ApiConfigDetail, PaginatedResponse, detail_response};
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
    };
    let mut configs = state.db.list_configs(&filter).await?;
    if let Some(protocol) = query.protocol.as_deref() {
        configs.retain(|config| config.protocol == protocol);
    }

    let total = configs.len();
    let start = ((page - 1) * per_page) as usize;
    let end = start.saturating_add(per_page as usize).min(total);
    let page_items = if start >= total {
        Vec::new()
    } else {
        configs[start..end].to_vec()
    };
    let mut items = Vec::with_capacity(page_items.len());
    for config in page_items {
        let latest_test = state.db.get_latest_connection_test(config.id).await?;
        items.push(detail_response(config, latest_test));
    }

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
    let config = state
        .db
        .get_config_by_id(id)
        .await?
        .ok_or(ServerError::NotFound)?;
    let latest_test = state.db.get_latest_connection_test(id).await?;

    Ok(Json(detail_response(config, latest_test)))
}
