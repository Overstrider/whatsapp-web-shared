//! Thin layer over `WaHandle` — exposes WA state/qr/reconnect to routes +
//! bridges `WaEvent` stream into the [`Broadcaster`] as domain `Event`s.

use tokio::sync::oneshot;

use crate::application::realtime::Broadcaster;
use crate::domain::events::Event;
use crate::domain::wa::WaConnState;
use crate::error::AppError;
use crate::infrastructure::wa_client::{WaCmd, WaEvent, WaHandle};

/// Orchestrator for the single global WA session.
#[derive(Clone)]
pub struct WaSessionService {
    handle: WaHandle,
    bc: Broadcaster,
}

impl WaSessionService {
    /// Wire the service + spawn the forwarding task that maps `WaEvent`
    /// frames into domain [`Event`]s on the shared broadcaster.
    pub fn new(handle: WaHandle, bc: Broadcaster) -> Self {
        let this = Self { handle, bc };
        this.spawn_forwarder();
        this
    }

    /// Cheap cached-state read — no DB, no await.
    pub fn state(&self) -> WaConnState {
        self.handle.state_snapshot()
    }

    /// Return the latest QR if the adapter is currently WaitingQr.
    pub fn qr(&self) -> Option<String> {
        self.handle.qr_snapshot()
    }

    /// Trigger a reconnect — fires a `WaCmd::Reconnect` and awaits ack.
    #[tracing::instrument(skip(self))]
    pub async fn reconnect(&self) -> Result<(), AppError> {
        let (tx, rx) = oneshot::channel();
        self.handle
            .cmd_tx
            .send(WaCmd::Reconnect { ack: tx })
            .await
            .map_err(|e| AppError::Wa(format!("cmd send: {e}")))?;
        rx.await
            .map_err(|e| AppError::Wa(format!("ack recv: {e}")))?
            .map_err(AppError::Wa)
    }

    fn spawn_forwarder(&self) {
        let mut rx = self.handle.subscribe();
        let bc = self.bc.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(WaEvent::StateChanged(s)) => {
                        bc.publish(Event::WaStateChange { state: s });
                    }
                    Ok(WaEvent::QrUpdated(q)) => {
                        bc.publish(Event::QrUpdate { qr: q });
                    }
                    Ok(WaEvent::MessageReceived { .. }) => {
                        // chat_service owns message persistence + publishing;
                        // it also subscribes to WaEvent and emits NewMessage.
                    }
                    Ok(WaEvent::ChatMetaUpdated { .. }) => {
                        // same — chat_service handles persistence/publish.
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(dropped = n, "wa forwarder lagged");
                        bc.publish(Event::WsLag { dropped: n });
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("wa forwarder: event channel closed");
                        break;
                    }
                }
            }
        });
    }
}
