//! Globally-shared `AppState` handed to every Axum handler via `State<AppState>`.
//!
//! Every field is cheap to clone (Arc/Pool/broadcast::Sender).

use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use sqlx::SqlitePool;

use crate::application::auth_service::AuthService;
use crate::application::chat_service::ChatService;
use crate::application::realtime::Broadcaster;
use crate::application::wa_session_service::WaSessionService;
use crate::config::Env;

/// Hand-off container carried by every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub cfg: Arc<Env>,
    pub auth: AuthService,
    pub wa: WaSessionService,
    pub chats: ChatService,
    pub broadcaster: Broadcaster,
    pub cookie_key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
