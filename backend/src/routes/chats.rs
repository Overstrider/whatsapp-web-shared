//! `/api/chats/*` — list chats, new chat, messages, send, reply, forward.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::routes::AuthUser;
use crate::state::AppState;

/// POST /api/chats body.
#[derive(Deserialize)]
pub struct NewChatReq {
    pub phone: String,
}

/// POST /api/chats/:id/messages body.
#[derive(Deserialize)]
pub struct SendReq {
    pub body: String,
}

/// POST /api/chats/:id/messages/:mid/reply body.
#[derive(Deserialize)]
pub struct ReplyReq {
    pub body: String,
}

/// POST /api/chats/:id/messages/:mid/forward body.
#[derive(Deserialize)]
pub struct ForwardReq {
    pub dst_chat_id: Uuid,
}

/// GET /api/chats/:id/messages query.
#[derive(Debug, Deserialize, Default)]
pub struct MessagesQuery {
    pub before: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<u32>,
}

/// GET /api/chats — all chats, most recent first.
#[tracing::instrument(skip(st, _user))]
pub async fn list(
    State(st): State<AppState>,
    _user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let chats = st.chats.list().await?;
    Ok(Json(chats))
}

/// POST /api/chats — create (or reuse) a chat for an E.164 phone.
#[tracing::instrument(skip(st, _user, body))]
pub async fn new_chat(
    State(st): State<AppState>,
    _user: AuthUser,
    Json(body): Json<NewChatReq>,
) -> Result<impl IntoResponse, AppError> {
    let chat = st.chats.new_chat(&body.phone).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

/// GET /api/chats/:id/messages — paged messages, newest first.
#[tracing::instrument(skip(st, _user))]
pub async fn messages(
    State(st): State<AppState>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<MessagesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let msgs = st
        .chats
        .messages(id, q.before, q.limit.unwrap_or(50))
        .await?;
    Ok(Json(msgs))
}

/// POST /api/chats/:id/messages — send an outbound text message.
#[tracing::instrument(skip(st, _user, body))]
pub async fn send(
    State(st): State<AppState>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SendReq>,
) -> Result<impl IntoResponse, AppError> {
    let msg = st.chats.send(id, &body.body).await?;
    Ok((StatusCode::CREATED, Json(msg)))
}

/// POST /api/chats/:id/messages/:mid/reply — reply to a specific message.
#[tracing::instrument(skip(st, _user, body))]
pub async fn reply(
    State(st): State<AppState>,
    _user: AuthUser,
    Path((id, mid)): Path<(Uuid, Uuid)>,
    Json(body): Json<ReplyReq>,
) -> Result<impl IntoResponse, AppError> {
    let msg = st.chats.reply(id, mid, &body.body).await?;
    Ok((StatusCode::CREATED, Json(msg)))
}

/// POST /api/chats/:id/messages/:mid/forward — copy msg into another chat.
#[tracing::instrument(skip(st, _user, body))]
pub async fn forward(
    State(st): State<AppState>,
    _user: AuthUser,
    Path((id, mid)): Path<(Uuid, Uuid)>,
    Json(body): Json<ForwardReq>,
) -> Result<impl IntoResponse, AppError> {
    let msg = st.chats.forward(id, mid, body.dst_chat_id).await?;
    Ok((StatusCode::CREATED, Json(msg)))
}
