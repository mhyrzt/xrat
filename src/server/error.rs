use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::server::response::ApiErrorResponse;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("missing api key")]
    MissingApiKey,
    #[error("invalid api key")]
    InvalidApiKey,
    #[error("invalid query: {0}")]
    InvalidQuery(String),
    #[error("config not found")]
    NotFound,
    #[error(transparent)]
    Database(#[from] crate::db::DbError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::MissingApiKey | Self::InvalidApiKey => StatusCode::UNAUTHORIZED,
            Self::InvalidQuery(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(ApiErrorResponse {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}
