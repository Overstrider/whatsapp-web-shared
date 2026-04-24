//! WhatsApp client adapter layer.
//!
//! Real implementation (`adapter.rs`) wraps the `whatsapp-rust` git crate and
//! pumps commands + events through a single owner task. The default build
//! uses [`StubWaClient`] — a deterministic fake — so `cargo check` works
//! offline without depending on the upstream crate.
//!
//! Swap to real by disabling default features + enabling `wa_real`.

use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwap;
use chrono::{DateTime, Utc};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::domain::wa::WaConnState;

pub mod stub;

/// Commands handed to the WA owner task. Each carries a oneshot ack.
#[derive(Debug)]
pub enum WaCmd {
    SendText {
        jid: String,
        body: String,
        reply_to_external_id: Option<String>,
        ack: oneshot::Sender<Result<String, String>>,
    },
    Reconnect {
        ack: oneshot::Sender<Result<(), String>>,
    },
    GetState {
        ack: oneshot::Sender<WaConnState>,
    },
}

/// Events emitted by the owner task, fan-out via broadcast.
#[derive(Debug, Clone)]
pub enum WaEvent {
    StateChanged(WaConnState),
    QrUpdated(String),
    MessageReceived {
        external_id: String,
        jid: String,
        body: String,
        ts: DateTime<Utc>,
        from_me: bool,
        reply_to_external_id: Option<String>,
    },
    ChatMetaUpdated {
        jid: String,
        display_name: String,
    },
}

/// Cheaply-cloneable handle passed into services + routes.
#[derive(Clone)]
pub struct WaHandle {
    pub cmd_tx: mpsc::Sender<WaCmd>,
    pub evt_tx: broadcast::Sender<WaEvent>,
    pub state: Arc<ArcSwap<WaConnState>>,
    pub qr: Arc<ArcSwap<Option<String>>>,
}

impl WaHandle {
    /// Subscribe to the event stream. Receiver is `Lagged(n)`-aware.
    pub fn subscribe(&self) -> broadcast::Receiver<WaEvent> {
        self.evt_tx.subscribe()
    }

    /// Snapshot the current cached state.
    pub fn state_snapshot(&self) -> WaConnState {
        (**self.state.load()).clone()
    }

    /// Snapshot the latest known QR (if any).
    pub fn qr_snapshot(&self) -> Option<String> {
        self.qr.load().as_ref().clone()
    }
}

/// Factory trait — one impl per backend (real / stub).
pub trait WaClient: Send + Sync {
    /// Spawn the owner task + return a handle. `session_dir` is where the
    /// adapter may persist its on-disk session blob.
    fn spawn(session_dir: PathBuf) -> anyhow::Result<WaHandle>;
}

/// Build the WA handle according to enabled features. Always compile-safe;
/// `wa_stub` default feature keeps this pointed at [`stub::StubWaClient`].
pub fn build_default(session_dir: PathBuf) -> anyhow::Result<WaHandle> {
    // Both `wa_stub` and the absence-of-real-adapter case resolve to the stub.
    stub::StubWaClient::spawn(session_dir)
}
