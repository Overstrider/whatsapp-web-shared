//! `/api/health` — liveness probe returning WA state alongside.

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use crate::domain::wa::WaConnState;
use crate::error::AppError;
use crate::state::AppState;

/// Health response body. `wa` is the current cached WA connection state.
#[derive(Serialize)]
pub struct HealthBody {
    pub ok: bool,
    pub wa: WaConnState,
}

/// GET /api/health — always 200 if the process can respond.
#[tracing::instrument(skip(st))]
pub async fn health(State(st): State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(Json(HealthBody {
        ok: true,
        wa: st.wa.state(),
    }))
}
