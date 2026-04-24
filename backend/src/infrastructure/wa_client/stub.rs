//! Deterministic fake WA client — default adapter for offline dev + CI.
//!
//! Behavior:
//! - On spawn: state starts `Disconnected`, transitions to `WaitingQr` after
//!   ~500ms, emits a fake QR payload. Does NOT advance to `Connected` on
//!   its own — a `Reconnect` cmd flips it to `Connected`.
//! - `SendText` returns a fake external_id, mirrors the body back as a
//!   synthetic `MessageReceived` event so chat-flow tests can exercise the
//!   full inbound loop.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use chrono::Utc;
use tokio::sync::{broadcast, mpsc};

use super::{WaClient, WaCmd, WaEvent, WaHandle};
use crate::domain::wa::WaConnState;

/// Zero-sized marker type. `spawn` is the real work.
pub struct StubWaClient;

impl WaClient for StubWaClient {
    fn spawn(_session_dir: PathBuf) -> anyhow::Result<WaHandle> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<WaCmd>(64);
        let (evt_tx, _evt_rx_dropped) = broadcast::channel::<WaEvent>(1024);
        let state = Arc::new(ArcSwap::from_pointee(WaConnState::Disconnected));
        let qr = Arc::new(ArcSwap::<Option<String>>::from_pointee(None));

        let state_cl = state.clone();
        let qr_cl = qr.clone();
        let evt_cl = evt_tx.clone();

        tokio::spawn(async move {
            // Give the HTTP stack a moment to come up before announcing.
            tokio::time::sleep(Duration::from_millis(500)).await;
            state_cl.store(Arc::new(WaConnState::WaitingQr));
            let _ = evt_cl.send(WaEvent::StateChanged(WaConnState::WaitingQr));
            let fake_qr =
                "STUB-QR:2@AbCdEf==,xyz,whatsapp-web-shared-stub".to_string();
            qr_cl.store(Arc::new(Some(fake_qr.clone())));
            let _ = evt_cl.send(WaEvent::QrUpdated(fake_qr));

            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    WaCmd::GetState { ack } => {
                        let _ = ack.send((**state_cl.load()).clone());
                    }
                    WaCmd::Reconnect { ack } => {
                        state_cl.store(Arc::new(WaConnState::Connected));
                        qr_cl.store(Arc::new(None));
                        let _ = evt_cl.send(WaEvent::StateChanged(WaConnState::Connected));
                        let _ = ack.send(Ok(()));
                    }
                    WaCmd::SendText {
                        jid,
                        body,
                        reply_to_external_id,
                        ack,
                    } => {
                        // Fake external id + echo as inbound so the full
                        // chat_service loop can be unit-tested end-to-end.
                        let external = format!("stub-{}", uuid_v4_short());
                        let _ = ack.send(Ok(external.clone()));
                        let _ = evt_cl.send(WaEvent::MessageReceived {
                            external_id: external,
                            jid,
                            body,
                            ts: Utc::now(),
                            from_me: true,
                            reply_to_external_id,
                        });
                    }
                }
            }
            tracing::info!("stub wa client: cmd channel closed, task exiting");
        });

        Ok(WaHandle {
            cmd_tx,
            evt_tx,
            state,
            qr,
        })
    }
}

fn uuid_v4_short() -> String {
    let u = uuid::Uuid::new_v4();
    u.simple().to_string()
}
