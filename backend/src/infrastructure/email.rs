//! Email sender trait + two implementations: stdout (dev/CI) and SMTP (prod).
//!
//! The SMTP path is TODO-level in this MVP skeleton — the type is wired up,
//! factory returns it, but the real `send_code` body just logs a warning and
//! falls back to stdout. Specialist finishes this later.

use async_trait::async_trait;

use crate::config::{EmailMode, Env};
use crate::error::AppError;

/// Abstract outbound magic-code mail transport.
#[async_trait]
pub trait EmailSender: Send + Sync {
    /// Deliver `code` to `to`. Must NOT log the code in production sinks;
    /// stdout sink obviously does for dev convenience.
    async fn send_code(&self, to: &str, code: &str) -> Result<(), AppError>;
}

/// Dev sink: prints `CODE email=... code=...` to stdout via `tracing`.
pub struct StdoutSender;

#[async_trait]
impl EmailSender for StdoutSender {
    async fn send_code(&self, to: &str, code: &str) -> Result<(), AppError> {
        // Deliberate: log at INFO so scripts can grep for `CODE email=`.
        tracing::info!(target: "magic_code", "CODE email={to} code={code}");
        Ok(())
    }
}

/// Real SMTP sender. TODO: wire `lettre` AsyncSmtpTransport here.
pub struct SmtpSender {
    pub from: String,
    // TODO(specialist): add AsyncSmtpTransport<Tokio1Executor> + build in factory.
}

#[async_trait]
impl EmailSender for SmtpSender {
    async fn send_code(&self, to: &str, code: &str) -> Result<(), AppError> {
        // MVP: log + fall through. Replace with real lettre transport.
        tracing::warn!(
            from = %self.from,
            to = %to,
            "SmtpSender::send_code is a TODO stub — logging instead of sending"
        );
        tracing::info!(target: "magic_code", "CODE email={to} code={code}");
        Ok(())
    }
}

/// Build the right [`EmailSender`] based on `cfg.email_mode`. If SMTP mode is
/// requested but config is incomplete we warn and return StdoutSender.
pub fn build(cfg: &Env) -> std::sync::Arc<dyn EmailSender> {
    match cfg.email_mode {
        EmailMode::Stdout => std::sync::Arc::new(StdoutSender),
        EmailMode::Smtp => {
            if cfg.smtp.host.is_empty() {
                tracing::warn!("EMAIL_MODE=smtp but SMTP_HOST empty — falling back to stdout");
                std::sync::Arc::new(StdoutSender)
            } else {
                std::sync::Arc::new(SmtpSender {
                    from: cfg.smtp.from.clone(),
                })
            }
        }
    }
}
