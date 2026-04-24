//! WhatsApp session-state domain.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// All possible states of the global WA session. Tagged serde for WS frames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "detail")]
pub enum WaConnState {
    Disconnected,
    WaitingQr,
    Connecting,
    Connected,
    LoggedOut,
    Error(String),
}

impl WaConnState {
    /// Short, DB-friendly tag (matches serde tag without payload).
    pub fn tag(&self) -> &'static str {
        match self {
            WaConnState::Disconnected => "Disconnected",
            WaConnState::WaitingQr => "WaitingQr",
            WaConnState::Connecting => "Connecting",
            WaConnState::Connected => "Connected",
            WaConnState::LoggedOut => "LoggedOut",
            WaConnState::Error(_) => "Error",
        }
    }
}

/// Opaque QR payload. Adapter chooses format (raw WA pairing string or
/// `data:image/png;base64,...`). Frontend just renders as-is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCode(pub String);

/// The singleton row in `wa_global_session`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSessionMeta {
    pub state: WaConnState,
    pub jid: Option<String>,
    pub last_qr: Option<String>,
    pub last_connected_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}
