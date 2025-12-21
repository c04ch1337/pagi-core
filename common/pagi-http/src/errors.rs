use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use pagi_common::{ErrorCode, PagiError};
use serde::Serialize;
use time::OffsetDateTime;

impl From<reqwest::Error> for PagiAxumError {
    fn from(value: reqwest::Error) -> Self {
        PagiError::from(value).into()
    }
}

impl From<std::io::Error> for PagiAxumError {
    fn from(value: std::io::Error) -> Self {
        PagiError::from(value).into()
    }
}

impl From<serde_json::Error> for PagiAxumError {
    fn from(value: serde_json::Error) -> Self {
        PagiError::from(value).into()
    }
}

/// Standard JSON error response body for all HTTP services/plugins.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub code: u32,
    pub timestamp: String,
}

/// Wraps [`PagiError`](common/pagi-common/src/lib.rs:25) to provide an Axum [`IntoResponse`] implementation.
///
/// This avoids Rust's orphan rules (Axum's trait + `pagi-common`'s type are both external to leaf crates).
#[derive(Debug)]
pub struct PagiAxumError {
    pub err: PagiError,
    /// Optional override for HTTP status code.
    pub status: Option<StatusCode>,
}

impl From<PagiError> for PagiAxumError {
    fn from(value: PagiError) -> Self {
        Self { err: value, status: None }
    }
}

impl PagiAxumError {
    pub fn with_status(err: PagiError, status: StatusCode) -> Self {
        Self {
            err,
            status: Some(status),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        if let Some(s) = self.status {
            return s;
        }
        match self.err.code() {
            ErrorCode::ConfigInvalid => StatusCode::BAD_REQUEST,
            ErrorCode::PluginLoadFailed => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::PluginExecutionFailed => StatusCode::BAD_GATEWAY,
            ErrorCode::NetworkTimeout => StatusCode::BAD_GATEWAY,
            ErrorCode::NetworkError => StatusCode::BAD_GATEWAY,
            ErrorCode::RedisError => StatusCode::BAD_GATEWAY,
            ErrorCode::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for PagiAxumError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.err.code() as u32;
        let timestamp = OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();

        let body = ErrorBody {
            error: self.err.to_string(),
            code,
            timestamp,
        };
        (status, Json(body)).into_response()
    }
}
