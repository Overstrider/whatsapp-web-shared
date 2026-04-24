//! Magic-code auth flow: request_code → verify_code → session_cookie.
//!
//! This service is fully implemented in the MVP (auth is on the critical path
//! for every other route). Argon2id hashes codes at rest; rate-limit rejects
//! abusive callers based on rolling 1-hour count per email.

use std::sync::Arc;

use argon2::password_hash::rand_core::OsRng as Argon2Rng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use rand::RngCore;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config::Env;
use crate::domain::auth::{AuthSession, MagicCode};
use crate::domain::user::{Email, User};
use crate::error::{AppError, AuthReason};
use crate::infrastructure::email::EmailSender;

/// Auth use-case orchestrator — DB + email transport only.
#[derive(Clone)]
pub struct AuthService {
    pool: SqlitePool,
    email: Arc<dyn EmailSender>,
    cfg: Arc<Env>,
}

impl AuthService {
    /// Construct a new service — cheap; clone freely.
    pub fn new(pool: SqlitePool, email: Arc<dyn EmailSender>, cfg: Arc<Env>) -> Self {
        Self { pool, email, cfg }
    }

    /// Issue a fresh 6-digit code, persist its argon2 hash, send via email.
    ///
    /// Enforces rolling-hour rate limit from `cfg.rate_limit_codes_per_hour`.
    #[tracing::instrument(skip(self), fields(email_hash = %hash_for_log(raw_email)))]
    pub async fn request_code(&self, raw_email: &str) -> Result<(), AppError> {
        let email = Email::parse(raw_email).map_err(AppError::Validation)?;
        let email_s = email.as_str();

        // Rate limit: count auth_codes in last hour for this email.
        let since = Utc::now()
            .checked_sub_signed(chrono::Duration::hours(1))
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("clock skew")))?
            .to_rfc3339();

        let recent: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM auth_codes WHERE email = ?1 AND created_at > ?2",
        )
        .bind(email_s)
        .bind(&since)
        .fetch_one(&self.pool)
        .await?;

        if recent as u32 >= self.cfg.rate_limit_codes_per_hour {
            tracing::warn!(email_hash = %hash_for_log(email_s), "rate limited");
            return Err(AppError::RateLimited);
        }

        let code = generate_magic_code();
        let code_hash = hash_code(code.expose())?;

        let id = Uuid::new_v4();
        let now = Utc::now();
        let expires = now
            + chrono::Duration::seconds(self.cfg.auth_code_ttl_secs as i64);

        sqlx::query(
            "INSERT INTO auth_codes (id, email, code_hash, expires_at, consumed, created_at) \
             VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        )
        .bind(id.to_string())
        .bind(email_s)
        .bind(&code_hash)
        .bind(expires.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.email.send_code(email_s, code.expose()).await?;
        Ok(())
    }

    /// Verify a user-entered code. On success, create + return a new session.
    ///
    /// On success the calling handler must set the HttpOnly cookie from the
    /// returned `token`.
    #[tracing::instrument(skip(self, raw_code), fields(email_hash = %hash_for_log(raw_email)))]
    pub async fn verify_code(
        &self,
        raw_email: &str,
        raw_code: &str,
    ) -> Result<(User, AuthSession), AppError> {
        let email = Email::parse(raw_email).map_err(AppError::Validation)?;
        let email_s = email.as_str();
        let code = MagicCode::parse(raw_code)
            .map_err(|_| AppError::Auth(AuthReason::InvalidCode))?;

        // Fetch newest unconsumed unexpired row for email.
        let now = Utc::now();
        let row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT id, code_hash, expires_at FROM auth_codes \
             WHERE email = ?1 AND consumed = 0 \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(email_s)
        .fetch_optional(&self.pool)
        .await?;

        let (id_s, hash, expires_s) = row.ok_or(AppError::Auth(AuthReason::InvalidCode))?;
        let expires: DateTime<Utc> = expires_s
            .parse()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("bad expires_at in db")))?;
        if expires < now {
            return Err(AppError::Auth(AuthReason::ExpiredCode));
        }

        if !verify_code_hash(code.expose(), &hash) {
            return Err(AppError::Auth(AuthReason::InvalidCode));
        }

        // Consume row.
        sqlx::query("UPDATE auth_codes SET consumed = 1 WHERE id = ?1")
            .bind(&id_s)
            .execute(&self.pool)
            .await?;

        // Upsert user by email.
        let user = upsert_user(&self.pool, email_s).await?;

        // Create session.
        let session = mint_session(&self.pool, user.id, &self.cfg).await?;

        Ok((user, session))
    }

    /// Look up the user behind a valid session token. Returns
    /// `Auth(NoSession|SessionExpired)` on miss or expired.
    #[tracing::instrument(skip(self, token))]
    pub async fn current_user(&self, token: &str) -> Result<User, AppError> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT s.user_id, s.expires_at FROM auth_sessions s WHERE s.token = ?1",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        let (user_id_s, expires_s) = row.ok_or(AppError::Auth(AuthReason::NoSession))?;
        let expires: DateTime<Utc> = expires_s
            .parse()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("bad session expires_at")))?;
        if expires < Utc::now() {
            return Err(AppError::Auth(AuthReason::SessionExpired));
        }

        let user_row: (String, String, String) = sqlx::query_as(
            "SELECT id, email, created_at FROM users WHERE id = ?1",
        )
        .bind(&user_id_s)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::Auth(AuthReason::NoSession))?;

        let (id, email, created_at) = user_row;
        Ok(User {
            id: Uuid::parse_str(&id).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
            email,
            created_at: created_at
                .parse()
                .map_err(|e: chrono::ParseError| AppError::Internal(anyhow::anyhow!(e)))?,
        })
    }

    /// Delete the session row (idempotent).
    #[tracing::instrument(skip(self, token))]
    pub async fn logout(&self, token: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM auth_sessions WHERE token = ?1")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// ---------- helpers ----------

fn generate_magic_code() -> MagicCode {
    // 6 ASCII digits from OsRng (uniform over 0..1_000_000).
    let mut rng = OsRng;
    let n = rng.next_u32() % 1_000_000;
    MagicCode::from_trusted(format!("{n:06}"))
}

fn hash_code(code: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut Argon2Rng);
    Argon2::default()
        .hash_password(code.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2 hash: {e}")))
}

fn verify_code_hash(code: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(code.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

async fn upsert_user(pool: &SqlitePool, email: &str) -> Result<User, AppError> {
    // Try fetch.
    let existing: Option<(String, String, String)> =
        sqlx::query_as("SELECT id, email, created_at FROM users WHERE email = ?1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

    if let Some((id, e, ca)) = existing {
        return Ok(User {
            id: Uuid::parse_str(&id).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
            email: e,
            created_at: ca
                .parse()
                .map_err(|e: chrono::ParseError| AppError::Internal(anyhow::anyhow!(e)))?,
        });
    }

    let id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query("INSERT INTO users (id, email, created_at) VALUES (?1, ?2, ?3)")
        .bind(id.to_string())
        .bind(email)
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

    Ok(User {
        id,
        email: email.to_string(),
        created_at: now,
    })
}

async fn mint_session(
    pool: &SqlitePool,
    user_id: Uuid,
    cfg: &Env,
) -> Result<AuthSession, AppError> {
    // 32 bytes OsRng → hex (64 chars).
    let mut buf = [0u8; 32];
    OsRng.fill_bytes(&mut buf);
    let token = hex::encode(buf);

    let now = Utc::now();
    let expires = now + chrono::Duration::days(cfg.session_ttl_days as i64);

    sqlx::query(
        "INSERT INTO auth_sessions (token, user_id, created_at, expires_at) \
         VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&token)
    .bind(user_id.to_string())
    .bind(now.to_rfc3339())
    .bind(expires.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(AuthSession {
        token,
        user_id,
        created_at: now,
        expires_at: expires,
    })
}

fn hash_for_log(email: &str) -> String {
    // Low-sensitivity trace tag — 8 hex of blake3 would be nice but we avoid
    // extra deps; simple truncated sha-like via stable hash from stdlib is
    // fine. We use a very weak xor-fold — only for log grouping, not security.
    let mut acc: u64 = 0xcbf29ce484222325;
    for b in email.as_bytes() {
        acc ^= *b as u64;
        acc = acc.wrapping_mul(0x100000001b3);
    }
    format!("{acc:016x}")
}

#[cfg(test)]
mod tests {
    use super::{generate_magic_code, hash_code, verify_code_hash};

    #[test]
    fn smoke_hash_roundtrip() {
        let code = generate_magic_code();
        let h = hash_code(code.expose()).unwrap();
        assert!(verify_code_hash(code.expose(), &h));
        assert!(!verify_code_hash("000000", &h) || code.expose() == "000000");
    }

    #[test]
    fn generated_code_is_six_digits() {
        let c = generate_magic_code();
        assert_eq!(c.expose().len(), 6);
        assert!(c.expose().chars().all(|ch| ch.is_ascii_digit()));
    }
}
