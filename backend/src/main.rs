//! Binary entry point.
//!
//! 1. dotenvy → tracing → env load.
//! 2. db init + migrations.
//! 3. wa client spawn (stub by default).
//! 4. router build + serve with graceful shutdown.

#![forbid(unsafe_code)]

use std::sync::Arc;

use axum_extra::extract::cookie::Key;
use tokio::signal;

use whatsapp_web_shared_backend::application::auth_service::AuthService;
use whatsapp_web_shared_backend::application::chat_service::ChatService;
use whatsapp_web_shared_backend::application::realtime::Broadcaster;
use whatsapp_web_shared_backend::application::wa_session_service::WaSessionService;
use whatsapp_web_shared_backend::build_app;
use whatsapp_web_shared_backend::config::Env;
use whatsapp_web_shared_backend::infrastructure::{db, email, session_store, wa_client};
use whatsapp_web_shared_backend::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    init_tracing();

    let cfg = Arc::new(Env::load()?);
    warn_if_cookie_key_weak(&cfg.cookie_key_hex);

    let pool = db::init(&cfg.database_url).await?;

    session_store::ensure_dir(&cfg.wa_session_path).await.ok();

    let wa_handle = wa_client::build_default(cfg.wa_session_path.clone())?;

    let bc = Broadcaster::new(1024);
    let wa = WaSessionService::new(wa_handle.clone(), bc.clone());
    let email_sender = email::build(&cfg);
    let auth = AuthService::new(pool.clone(), email_sender, cfg.clone());
    let chats = ChatService::new(pool.clone(), wa_handle, bc.clone());

    // Derive signing key from 64-hex (32 bytes).
    let cookie_key = build_cookie_key(&cfg.cookie_key_hex)?;

    let state = AppState {
        pool,
        cfg: cfg.clone(),
        auth,
        wa,
        chats,
        broadcaster: bc,
        cookie_key,
    };

    let app = build_app(state);
    let bind_addr = cfg.bind_addr;
    tracing::info!(%bind_addr, "listening");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,whatsapp_web_shared_backend=debug"));
    fmt().with_env_filter(filter).compact().init();
}

fn warn_if_cookie_key_weak(hex_key: &str) {
    if hex_key.len() != 64 {
        tracing::error!(
            len = hex_key.len(),
            "COOKIE_KEY must be 64 hex chars (32 bytes). Set via `openssl rand -hex 32`."
        );
    } else if hex_key.chars().all(|c| c == '0') {
        tracing::warn!("COOKIE_KEY is the all-zero dev default — rotate before prod.");
    }
}

fn build_cookie_key(hex_key: &str) -> anyhow::Result<Key> {
    // `Key` requires ≥ 64 bytes of signing material. We expand the 32-byte
    // user-supplied secret via SHA-like simple fold: duplicate the 32 bytes
    // twice to reach 64. Good enough for signed-not-encrypted dev; specialist
    // should replace with HKDF when promoting to prod.
    let bytes = hex::decode(hex_key).map_err(|e| anyhow::anyhow!("COOKIE_KEY hex: {e}"))?;
    if bytes.len() != 32 {
        anyhow::bail!("COOKIE_KEY must decode to 32 bytes");
    }
    let mut buf = Vec::with_capacity(64);
    buf.extend_from_slice(&bytes);
    buf.extend_from_slice(&bytes);
    Ok(Key::from(&buf))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut s) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
    tracing::info!("shutdown signal received");
}
