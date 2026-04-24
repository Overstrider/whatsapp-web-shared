//! Message + Direction.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Direction of a message relative to the account bound to the WA session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    In,
    Out,
}

impl Direction {
    /// Stable DB string ("in"|"out"). Matches CHECK constraint in 0001_init.
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::In => "in",
            Direction::Out => "out",
        }
    }

    /// Reverse of [`Direction::as_str`] — used when loading rows.
    pub fn from_db(s: &str) -> Result<Self, String> {
        match s {
            "in" => Ok(Direction::In),
            "out" => Ok(Direction::Out),
            other => Err(format!("invalid direction tag {other:?}")),
        }
    }
}

/// A persisted chat message (text-only in v1). `body` max 65_535 bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub chat_id: Uuid,
    /// WA upstream message id. UNIQUE, used to dedup inbound events.
    pub external_id: Option<String>,
    pub direction: Direction,
    pub body: String,
    pub reply_to_message_id: Option<Uuid>,
    pub forwarded_from_message_id: Option<Uuid>,
    pub ts: DateTime<Utc>,
}
