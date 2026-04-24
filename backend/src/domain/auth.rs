//! Auth-flow domain types: magic codes + server sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Six-digit ASCII magic code. Display impl redacts to `******`.
#[derive(Clone, PartialEq, Eq)]
pub struct MagicCode(String);

impl MagicCode {
    /// Validate + wrap a raw 6-digit code string.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let s = raw.trim();
        if s.len() != 6 || !s.chars().all(|c| c.is_ascii_digit()) {
            return Err("code must be exactly 6 digits".into());
        }
        Ok(MagicCode(s.to_string()))
    }

    /// Build from a freshly-generated code string, skipping validation.
    ///
    /// Internal constructor for generators that already produced 6 digits.
    pub(crate) fn from_trusted(s: String) -> Self {
        MagicCode(s)
    }

    /// Borrow the raw digits (use ONLY for hashing + email send — never log).
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for MagicCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MagicCode(******)")
    }
}

impl std::fmt::Display for MagicCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("******")
    }
}

/// DB-persisted pending code record.
#[derive(Debug, Clone)]
pub struct AuthCode {
    pub id: Uuid,
    pub email: String,
    /// argon2id-hashed code digits.
    pub code_hash: String,
    pub expires_at: DateTime<Utc>,
    pub consumed: bool,
    pub created_at: DateTime<Utc>,
}

/// DB-persisted server session row.
///
/// `token` is 32 bytes of OsRng → hex (64 chars); serves as cookie value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub token: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::MagicCode;

    #[test]
    fn accepts_six_digits() {
        assert!(MagicCode::parse("123456").is_ok());
    }

    #[test]
    fn rejects_too_short() {
        assert!(MagicCode::parse("12345").is_err());
    }

    #[test]
    fn rejects_non_digits() {
        assert!(MagicCode::parse("12345a").is_err());
    }

    #[test]
    fn debug_redacts() {
        let c = MagicCode::parse("123456").unwrap();
        assert_eq!(format!("{c:?}"), "MagicCode(******)");
    }
}
