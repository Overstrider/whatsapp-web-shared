//! `/api/auth/*` handlers: magic-code start + verify, logout, session.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use serde::{Deserialize, Serialize};

use crate::domain::user::User;
use crate::error::AppError;
use crate::routes::AuthUser;
use crate::state::AppState;

/// `POST /api/auth/login/start` body.
#[derive(Deserialize)]
pub struct LoginStartReq {
    pub email: String,
}

/// `POST /api/auth/login/verify` body.
#[derive(Deserialize)]
pub struct LoginVerifyReq {
    pub email: String,
    pub code: String,
}

/// Wrapper serialized into most auth endpoint responses.
#[derive(Serialize)]
pub struct UserBody {
    pub user: User,
}

/// POST /api/auth/login/start — issues a magic code via email sink.
#[tracing::instrument(skip(st, body))]
pub async fn login_start(
    State(st): State<AppState>,
    Json(body): Json<LoginStartReq>,
) -> Result<impl IntoResponse, AppError> {
    st.auth.request_code(&body.email).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/auth/login/verify — verify code, create session, set cookie.
#[tracing::instrument(skip(st, jar, body))]
pub async fn login_verify(
    State(st): State<AppState>,
    jar: PrivateCookieJar,
    Json(body): Json<LoginVerifyReq>,
) -> Result<impl IntoResponse, AppError> {
    let (user, session) = st.auth.verify_code(&body.email, &body.code).await?;

    let secure = st.cfg.cookie_secure;
    // axum-extra re-exports the `cookie` crate which internally uses `time`.
    // Construct Max-Age via seconds to avoid pulling `time` as direct dep.
    let secs = (st.cfg.session_ttl_days as i64) * 24 * 3600;
    let mut c = Cookie::new("sid", session.token);
    c.set_http_only(true);
    c.set_secure(secure);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");
    c.set_max_age(time::Duration::seconds(secs));

    let jar = jar.add(c);
    Ok((jar, Json(UserBody { user })))
}

/// POST /api/auth/logout — remove server session + clear cookie.
#[tracing::instrument(skip(st, jar))]
pub async fn logout(
    State(st): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, AppError> {
    if let Some(c) = jar.get("sid") {
        st.auth.logout(c.value()).await?;
    }
    let jar = jar.remove(Cookie::from("sid"));
    Ok((jar, StatusCode::NO_CONTENT))
}

/// GET /api/auth/session — returns current user if authenticated, else 401.
#[tracing::instrument(skip(_st, user))]
pub async fn session(
    State(_st): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(UserBody { user }))
}

