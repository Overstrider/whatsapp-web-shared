//! User entity + validated `Email` newtype.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A registered user — created on first successful magic-code verify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

/// Wrapper around a normalized email string.
///
/// Construction enforces: non-empty, contains `@`, length ≤ 320, lowercased.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Validate + normalize a raw email string.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let trimmed = raw.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            return Err("email must not be empty".into());
        }
        if trimmed.len() > 320 {
            return Err("email too long (>320 chars)".into());
        }
        if !trimmed.contains('@') {
            return Err("email must contain '@'".into());
        }
        Ok(Email(trimmed))
    }

    /// Borrow the normalized inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume into the underlying `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Email;

    #[test]
    fn accepts_simple_email() {
        assert!(Email::parse("foo@bar.com").is_ok());
    }

    #[test]
    fn normalizes_case_and_whitespace() {
        let e = Email::parse("  Foo@BAR.com  ").unwrap();
        assert_eq!(e.as_str(), "foo@bar.com");
    }

    #[test]
    fn rejects_missing_at() {
        assert!(Email::parse("noat").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(Email::parse("   ").is_err());
    }
}
