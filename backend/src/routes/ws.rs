//! WebSocket upgrade — forwards broadcaster events to the client as JSON.
//!
//! Auth: reads signed `sid` cookie on the upgrade request. If missing/invalid
//! the upgrade still completes at the HTTP layer, then we immediately close
//! the socket with code 4401. Browsers cannot read HTTP status from failed
//! upgrades, so this close-code approach is the only reliable signal.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::PrivateCookieJar;

use crate::domain::events::Event;
use crate::error::AppError;
use crate::state::AppState;

/// GET /ws — HTTP→WS upgrade entry point.
#[tracing::instrument(skip(st, jar, upgrade))]
pub async fn upgrade(
    State(st): State<AppState>,
    jar: PrivateCookieJar,
    upgrade: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    // Validate session at upgrade time — DO NOT fail the HTTP handshake
    // (browser can't see it). We let it upgrade then close with 4401.
    let token = jar.get("sid").map(|c| c.value().to_string());
    let user_ok = if let Some(t) = token.as_ref() {
        st.auth.current_user(t).await.is_ok()
    } else {
        false
    };

    let bc = st.broadcaster.clone();
    Ok(upgrade.on_upgrade(move |socket| handle_socket(socket, bc, user_ok)))
}

async fn handle_socket(
    mut socket: WebSocket,
    bc: crate::application::realtime::Broadcaster,
    authed: bool,
) {
    if !authed {
        let _ = socket
            .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                code: 4401,
                reason: "unauthenticated".into(),
            })))
            .await;
        return;
    }

    let mut rx = bc.subscribe();

    loop {
        tokio::select! {
            maybe_event = rx.recv() => {
                match maybe_event {
                    Ok(ev) => {
                        if let Ok(s) = serde_json::to_string(&ev) {
                            if socket.send(Message::Text(s.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        let lag = Event::WsLag { dropped: n };
                        if let Ok(s) = serde_json::to_string(&lag) {
                            let _ = socket.send(Message::Text(s.into())).await;
                        }
                    }
                    Err(_) => break,
                }
            }
            maybe_msg = socket.recv() => {
                match maybe_msg {
                    Some(Ok(Message::Ping(p))) => { let _ = socket.send(Message::Pong(p)).await; }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => { tracing::debug!(error = %e, "ws recv error"); break; }
                    _ => {}
                }
            }
        }
    }
}
