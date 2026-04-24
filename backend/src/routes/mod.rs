//! Router assembly + `require_session` middleware extractor.

use std::time::Duration;

use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::Router;
use axum_extra::extract::cookie::{Key, PrivateCookieJar};
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::domain::user::User;
use crate::error::{AppError, AuthReason};
use crate::state::AppState;

pub mod auth;
pub mod chats;
pub mod health;
pub mod whatsapp;
pub mod ws;

/// Assemble the top-level Axum `Router` from [`AppState`].
pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route(
            "/auth/login/start",
            axum::routing::post(auth::login_start),
        )
        .route(
            "/auth/login/verify",
            axum::routing::post(auth::login_verify),
        )
        .route("/auth/logout", axum::routing::post(auth::logout))
        .route("/auth/session", axum::routing::get(auth::session))
        .route("/whatsapp/state", axum::routing::get(whatsapp::state))
        .route("/whatsapp/qr", axum::routing::get(whatsapp::qr))
        .route(
            "/whatsapp/reconnect",
            axum::routing::post(whatsapp::reconnect),
        )
        .route(
            "/chats",
            axum::routing::get(chats::list).post(chats::new_chat),
        )
        .route(
            "/chats/:id/messages",
            axum::routing::get(chats::messages).post(chats::send),
        )
        .route(
            "/chats/:id/messages/:mid/reply",
            axum::routing::post(chats::reply),
        )
        .route(
            "/chats/:id/messages/:mid/forward",
            axum::routing::post(chats::forward),
        )
        .route("/health", axum::routing::get(health::health));

    let cors = CorsLayer::new()
        .allow_origin(
            state
                .cfg
                .frontend_origin
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:3000")),
        )
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .nest("/api", api)
        .route("/ws", axum::routing::get(ws::upgrade))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(30)))
        .with_state(state)
}

// ---------- require_session extractor ----------

/// `FromRequestParts` that reads the signed `sid` cookie, looks up the session
/// row, and returns the [`User`] (also attaches it to request extensions for
/// downstream handlers that prefer `Extension<User>`).
pub struct AuthUser(pub User);

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
    Key: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app = AppState::from_ref(state);
        let jar: PrivateCookieJar = PrivateCookieJar::from_headers(&parts.headers, app.cookie_key.clone());

        let token = jar
            .get("sid")
            .map(|c| c.value().to_string())
            .ok_or(AppError::Auth(AuthReason::NoSession))?;

        let user = app.auth.current_user(&token).await?;
        parts.extensions.insert(user.clone());
        Ok(AuthUser(user))
    }
}

// Small 204 helper — used by login_start + logout + reconnect 202.
pub fn no_content() -> StatusCode {
    StatusCode::NO_CONTENT
}

/// Re-export for handlers that want raw `State<AppState>`.
pub type S<'a> = State<AppState>;
