//! Application-wide error type + JSON IntoResponse mapping.
//!
//! Handlers return `Result<T, AppError>` exclusively; Axum uses the
//! `IntoResponse` impl below to serialize a stable error envelope to clients.
//!
//! Inner strings of `Db` / `Internal` variants are NEVER returned to clients —
//! they're logged via `tracing::error!` and a safe `code` string is sent instead.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// The top-level error enum every handler + service returns.
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    /// Anything coming out of sqlx — row not found is mapped separately.
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    /// Upstream WA adapter failure (disconnected, timeout, protocol).
    #[error("wa: {0}")]
    Wa(String),
    /// Auth-flow specific refusals (see [`AuthReason`]).
    #[error("auth: {0:?}")]
    Auth(AuthReason),
    /// Caller-supplied data failed validation — message is safe to show.
    #[error("validation: {0}")]
    Validation(String),
    /// Email send-path failure — message is NOT returned to client.
    #[error("email: {0}")]
    Email(String),
    /// Row/entity not found.
    #[error("not_found")]
    NotFound,
    /// Unique-constraint / state-transition conflict.
    #[error("conflict: {0}")]
    Conflict(String),
    /// Rate-limit bucket exhausted.
    #[error("rate_limited")]
    RateLimited,
    /// Catch-all for unexpected internal errors.
    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Specific reasons the auth flow rejected a request, mapped to HTTP 400/401.
#[derive(Debug, Clone, Copy)]
pub enum AuthReason {
    /// Code did not match any active hash, or email typo.
    InvalidCode,
    /// Code matched but its TTL has passed.
    ExpiredCode,
    /// No `sid` cookie, or cookie signature bad.
    NoSession,
    /// Cookie valid but DB session row expired.
    SessionExpired,
}

/// Wire shape every error returns. `message` only present for `Validation`.
#[derive(Serialize)]
struct ErrorBody<'a> {
    error: ErrorInner<'a>,
}

#[derive(Serialize)]
struct ErrorInner<'a> {
    code: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Choose (status, code, optional safe message) per variant.
        let (status, code, message): (StatusCode, &str, Option<String>) = match &self {
            AppError::Db(e) => {
                tracing::error!(error = ?e, "db error");
                (StatusCode::INTERNAL_SERVER_ERROR, "db_error", None)
            }
            AppError::Wa(e) => {
                tracing::warn!(error = %e, "wa error");
                (StatusCode::BAD_GATEWAY, "wa_unavailable", None)
            }
            AppError::Auth(reason) => match reason {
                AuthReason::InvalidCode | AuthReason::ExpiredCode => {
                    (StatusCode::BAD_REQUEST, "invalid_code", None)
                }
                AuthReason::NoSession | AuthReason::SessionExpired => {
                    (StatusCode::UNAUTHORIZED, "unauthenticated", None)
                }
            },
            AppError::Validation(m) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", Some(m.clone()))
            }
            AppError::Email(e) => {
                tracing::error!(error = %e, "email error");
                (StatusCode::INTERNAL_SERVER_ERROR, "email_failed", None)
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found", None),
            AppError::Conflict(m) => (StatusCode::CONFLICT, "conflict", Some(m.clone())),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited", None),
            AppError::Internal(e) => {
                tracing::error!(error = ?e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", None)
            }
        };

        let body = ErrorBody {
            error: ErrorInner { code, message },
        };
        (status, Json(body)).into_response()
    }
}
