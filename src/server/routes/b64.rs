use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, Response, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::json::JsonQuery;
use crate::db::ConfigListFilter;
use crate::server::auth::require_api_key;
use crate::server::{ServerError, ServerResult, ServerState};

pub async fn b64(
    State(state): State<ServerState>,
    Query(query): Query<JsonQuery>,
) -> ServerResult<Response<Body>> {
    require_api_key(&state, query.key.as_deref())?;
    let filter = ConfigListFilter {
        only_enabled: query.enabled.unwrap_or(true),
        only_active: false,
        only_deleted: false,
        include_deleted: false,
        subscription_id: None,
        protocol: query.protocol.clone(),
    };

    let raw_configs: Vec<String> = if let Some(top) = query.top {
        if top == 0 || top > 200 {
            return Err(ServerError::InvalidQuery(
                "top must be between 1 and 200".to_string(),
            ));
        }
        let rows = state
            .db
            .list_top_configs_by_real_delay(top as i64, &filter)
            .await?;
        rows.into_iter().map(|row| row.config.raw_config).collect()
    } else {
        let configs = state.db.list_configs(&filter).await?;
        configs
            .into_iter()
            .map(|config| config.raw_config)
            .collect()
    };

    let payload = raw_configs
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let encoded = STANDARD.encode(payload);
    let mut response = Response::new(Body::from(encoded));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    Ok(response)
}
