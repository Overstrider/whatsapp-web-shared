//! In-process event fan-out for WebSocket subscribers.

use tokio::sync::broadcast;

use crate::domain::events::Event;

/// Cloneable broadcast wrapper. `Sender` is intentionally cheap to clone.
#[derive(Clone)]
pub struct Broadcaster {
    tx: broadcast::Sender<Event>,
}

impl Broadcaster {
    /// Build a new broadcaster with the given buffer capacity.
    pub fn new(cap: usize) -> Self {
        let (tx, _rx) = broadcast::channel(cap);
        Self { tx }
    }

    /// Subscribe a new receiver — one per WS connection.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    /// Publish an event; dropped silently if there are zero receivers.
    pub fn publish(&self, e: Event) {
        if let Err(err) = self.tx.send(e) {
            tracing::trace!(error = %err, "broadcaster publish: no active receivers");
        }
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new(1024)
    }
}
