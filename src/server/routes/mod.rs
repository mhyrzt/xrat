mod b64;
mod configs;
mod health;
mod json;

use axum::Router;
use axum::routing::get;

use crate::server::ServerState;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/json", get(json::json))
        .route("/b64", get(b64::b64))
        .route("/configs", get(configs::list_configs))
        .route("/configs/{id}", get(configs::get_config))
}
