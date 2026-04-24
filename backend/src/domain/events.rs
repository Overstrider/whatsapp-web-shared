//! Real-time events pushed to WS subscribers. JSON-tagged by `type`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::chat::Chat;
use super::message::Message;
use super::wa::WaConnState;

/// Every event that fans out to every connected WS client (global session).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// WA connection state transition.
    WaStateChange { state: WaConnState },
    /// Fresh QR pairing payload, emitted while WaitingQr.
    QrUpdate { qr: String },
    /// A new message landed — inbound from WA or echo of our own send.
    NewMessage { chat_id: Uuid, message: Message },
    /// A chat was created or renamed.
    ChatUpserted { chat: Chat },
    /// Broadcast channel lag — client should re-fetch via REST.
    WsLag { dropped: u64 },
}
