//! Chat + Jid domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// WhatsApp JID (Jabber ID). Always `user@s.whatsapp.net` or `group@g.us`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Jid(String);

impl Jid {
    /// Validate + wrap a raw JID string.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let s = raw.trim();
        if s.is_empty() {
            return Err("jid must not be empty".into());
        }
        if !s.contains('@') {
            return Err("jid must contain '@'".into());
        }
        Ok(Jid(s.to_string()))
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume into the underlying `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// A chat row as stored in `chats`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: Uuid,
    pub wa_jid: String,
    pub display_name: String,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::Jid;

    #[test]
    fn accepts_user_jid() {
        assert!(Jid::parse("1234567890@s.whatsapp.net").is_ok());
    }

    #[test]
    fn rejects_no_at() {
        assert!(Jid::parse("1234567890").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(Jid::parse("  ").is_err());
    }
}
