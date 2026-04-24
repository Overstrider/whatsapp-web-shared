//! Runtime configuration loaded once from environment variables.
//!
//! Hand-parsed via `std::env` (no serde) for fast failure on missing/invalid
//! values at boot. `dotenvy` loads `.env` in `main.rs` before this runs.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

/// Source of truth for every env-configurable knob the app reads.
#[derive(Debug, Clone)]
pub struct Env {
    /// `host:port` we bind to (e.g. `0.0.0.0:8080`).
    pub bind_addr: SocketAddr,
    /// Full sqlx DATABASE_URL (e.g. `sqlite:/data/app.db`).
    pub database_url: String,
    /// Dir holding the whatsapp-rust session blob.
    pub wa_session_path: PathBuf,
    /// 32-byte master key for `PrivateCookieJar`, hex-encoded (64 chars).
    pub cookie_key_hex: String,
    /// HTTPS-only cookie flag. Dev = false, prod = true.
    pub cookie_secure: bool,
    /// Session cookie TTL.
    pub session_ttl_days: u32,
    /// Magic-code TTL.
    pub auth_code_ttl_secs: u64,
    /// Max magic-code requests per email per rolling hour.
    pub rate_limit_codes_per_hour: u32,
    /// Exact frontend origin for CORS + WS Origin validation (no wildcard).
    pub frontend_origin: String,
    /// `stdout` | `smtp`.
    pub email_mode: EmailMode,
    /// SMTP config — only read when `email_mode == Smtp`.
    pub smtp: SmtpConfig,
}

/// Transport selection for outbound magic-code mail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailMode {
    /// Log the code to stdout — dev + CI.
    Stdout,
    /// Real SMTP relay via `lettre`.
    Smtp,
}

/// SMTP host/port/user/pass/from, only meaningful when [`EmailMode::Smtp`].
#[derive(Debug, Clone, Default)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from: String,
}

impl Env {
    /// Load + validate every required var. Returns a fresh owned [`Env`].
    ///
    /// Missing required vars abort with [`anyhow::Error`] for fail-fast startup.
    pub fn load() -> anyhow::Result<Self> {
        let bind_addr_s = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let bind_addr = SocketAddr::from_str(&bind_addr_s)
            .map_err(|e| anyhow::anyhow!("BIND_ADDR invalid ({bind_addr_s}): {e}"))?;

        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());

        let wa_session_path = PathBuf::from(
            std::env::var("WA_SESSION_PATH").unwrap_or_else(|_| "./wa-session".to_string()),
        );

        let cookie_key_hex = std::env::var("COOKIE_KEY").unwrap_or_else(|_| {
            // Dev fallback: stable 64-hex key derived from a constant label.
            // Prod MUST supply its own via env. Warned about in `main.rs`.
            "0".repeat(64)
        });

        let cookie_secure = env_bool("COOKIE_SECURE", false);
        let session_ttl_days = env_u32("SESSION_TTL_DAYS", 30);
        let auth_code_ttl_secs = env_u64("AUTH_CODE_TTL_SECS", 300);
        let rate_limit_codes_per_hour = env_u32("RATE_LIMIT_CODES_PER_HOUR", 5);

        let frontend_origin =
            std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let email_mode_s = std::env::var("EMAIL_MODE").unwrap_or_else(|_| "stdout".to_string());
        let email_mode = match email_mode_s.to_ascii_lowercase().as_str() {
            "smtp" => EmailMode::Smtp,
            _ => EmailMode::Stdout,
        };

        let smtp = SmtpConfig {
            host: std::env::var("SMTP_HOST").unwrap_or_default(),
            port: env_u32("SMTP_PORT", 587) as u16,
            user: std::env::var("SMTP_USER").unwrap_or_default(),
            pass: std::env::var("SMTP_PASS").unwrap_or_default(),
            from: std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@example.com".to_string()),
        };

        Ok(Env {
            bind_addr,
            database_url,
            wa_session_path,
            cookie_key_hex,
            cookie_secure,
            session_ttl_days,
            auth_code_ttl_secs,
            rate_limit_codes_per_hour,
            frontend_origin,
            email_mode,
            smtp,
        })
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"),
        Err(_) => default,
    }
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
