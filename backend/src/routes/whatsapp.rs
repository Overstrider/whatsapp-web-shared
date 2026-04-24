//! `/api/whatsapp/*` — session state, QR pairing, reconnect.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use crate::domain::wa::WaConnState;
use crate::error::AppError;
use crate::routes::AuthUser;
use crate::state::AppState;

/// Response for `GET /api/whatsapp/state`.
#[derive(Serialize)]
pub struct StateBody {
    pub state: WaConnState,
}

/// Response for `GET /api/whatsapp/qr`.
#[derive(Serialize)]
pub struct QrBody {
    pub qr: String,
}

/// GET /api/whatsapp/state — cached snapshot of WA connection state.
#[tracing::instrument(skip(st, _user))]
pub async fn state(
    State(st): State<AppState>,
    _user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(StateBody { state: st.wa.state() }))
}

/// GET /api/whatsapp/qr — current pairing QR. 409 if already Connected.
#[tracing::instrument(skip(st, _user))]
pub async fn qr(
    State(st): State<AppState>,
    _user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    if matches!(st.wa.state(), WaConnState::Connected) {
        return Err(AppError::Conflict("already connected".into()));
    }
    match st.wa.qr() {
        Some(qr) => Ok(Json(QrBody { qr })),
        None => Err(AppError::NotFound),
    }
}

/// POST /api/whatsapp/reconnect — triggers a reconnect cmd, returns 202.
#[tracing::instrument(skip(st, _user))]
pub async fn reconnect(
    State(st): State<AppState>,
    _user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    st.wa.reconnect().await?;
    Ok(StatusCode::ACCEPTED)
}
