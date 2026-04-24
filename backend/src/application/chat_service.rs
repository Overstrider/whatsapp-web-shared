//! Chat + message use cases. v1 text-only.
//!
//! MVP simplifications vs final plan:
//! - `list`, `messages`, `new_chat`, `send` write/read DB rows directly.
//! - `send` invokes the WA handle but does NOT wait for WA ack before DB
//!   insert; it best-effort fires the command. Proper ack+retry left as TODO.
//! - `reply` / `forward` persist a DB row but WA dispatch is a TODO.

use chrono::Utc;
use sqlx::SqlitePool;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::application::realtime::Broadcaster;
use crate::domain::chat::{Chat, Jid};
use crate::domain::events::Event;
use crate::domain::message::{Direction, Message};
use crate::error::AppError;
use crate::infrastructure::wa_client::{WaCmd, WaHandle};

/// Max bytes we accept for a message body.
pub const MAX_BODY_BYTES: usize = 65_535;

/// Orchestrator for chat + message flows.
#[derive(Clone)]
pub struct ChatService {
    pool: SqlitePool,
    handle: WaHandle,
    bc: Broadcaster,
}

impl ChatService {
    /// Wire the service.
    pub fn new(pool: SqlitePool, handle: WaHandle, bc: Broadcaster) -> Self {
        Self { pool, handle, bc }
    }

    /// List all chats, most-recent-activity first.
    #[tracing::instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<Chat>, AppError> {
        let rows: Vec<(String, String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT id, wa_jid, display_name, last_message_at, created_at FROM chats \
             ORDER BY COALESCE(last_message_at, created_at) DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(id, jid, name, last, created)| {
                Ok(Chat {
                    id: Uuid::parse_str(&id)
                        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
                    wa_jid: jid,
                    display_name: name,
                    last_message_at: last
                        .map(|s| s.parse())
                        .transpose()
                        .map_err(|e: chrono::ParseError| AppError::Internal(anyhow::anyhow!(e)))?,
                    created_at: created.parse().map_err(|e: chrono::ParseError| {
                        AppError::Internal(anyhow::anyhow!(e))
                    })?,
                })
            })
            .collect()
    }

    /// Fetch a page of messages for a chat, newest first.
    #[tracing::instrument(skip(self))]
    pub async fn messages(
        &self,
        chat_id: Uuid,
        _before: Option<chrono::DateTime<chrono::Utc>>,
        limit: u32,
    ) -> Result<Vec<Message>, AppError> {
        let limit = limit.clamp(1, 200) as i64;
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
        )> = sqlx::query_as(
            "SELECT id, chat_id, external_id, direction, body, \
                    reply_to_message_id, forwarded_from_message_id, ts \
             FROM messages WHERE chat_id = ?1 \
             ORDER BY ts DESC LIMIT ?2",
        )
        .bind(chat_id.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_message)
            .collect::<Result<Vec<_>, _>>()
    }

    /// Create a chat for a freshly-typed phone number.
    ///
    /// Validates via `phonenumber`, builds `<E164 digits>@s.whatsapp.net`.
    #[tracing::instrument(skip(self))]
    pub async fn new_chat(&self, phone_e164: &str) -> Result<Chat, AppError> {
        use phonenumber::parse;
        let parsed = parse(None, phone_e164)
            .map_err(|e| AppError::Validation(format!("phone: {e}")))?;
        if !phonenumber::is_valid(&parsed) {
            return Err(AppError::Validation("phone not valid".into()));
        }
        // Canonical E.164 without '+' — typical WA JID user part.
        let e164 = parsed
            .format()
            .mode(phonenumber::Mode::E164)
            .to_string()
            .trim_start_matches('+')
            .to_string();

        let jid = Jid::parse(&format!("{e164}@s.whatsapp.net"))
            .map_err(AppError::Validation)?;
        let display = format!("+{e164}");

        // Upsert by wa_jid.
        let existing: Option<(String, String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT id, wa_jid, display_name, last_message_at, created_at FROM chats WHERE wa_jid = ?1",
        )
        .bind(jid.as_str())
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, j, name, last, created)) = existing {
            return Ok(Chat {
                id: Uuid::parse_str(&id)
                    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
                wa_jid: j,
                display_name: name,
                last_message_at: last
                    .map(|s| s.parse())
                    .transpose()
                    .map_err(|e: chrono::ParseError| AppError::Internal(anyhow::anyhow!(e)))?,
                created_at: created
                    .parse()
                    .map_err(|e: chrono::ParseError| AppError::Internal(anyhow::anyhow!(e)))?,
            });
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO chats (id, wa_jid, display_name, last_message_at, created_at) \
             VALUES (?1, ?2, ?3, NULL, ?4)",
        )
        .bind(id.to_string())
        .bind(jid.as_str())
        .bind(&display)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        let chat = Chat {
            id,
            wa_jid: jid.into_inner(),
            display_name: display,
            last_message_at: None,
            created_at: now,
        };
        self.bc.publish(Event::ChatUpserted { chat: chat.clone() });
        Ok(chat)
    }

    /// Persist + best-effort dispatch an outbound text message.
    #[tracing::instrument(skip(self, body))]
    pub async fn send(&self, chat_id: Uuid, body: &str) -> Result<Message, AppError> {
        validate_body(body)?;
        let chat = self.find_chat(chat_id).await?;

        // Fire WA send command — don't block DB insert on ack in MVP.
        let (tx, _rx) = oneshot::channel();
        let _ = self
            .handle
            .cmd_tx
            .send(WaCmd::SendText {
                jid: chat.wa_jid.clone(),
                body: body.to_string(),
                reply_to_external_id: None,
                ack: tx,
            })
            .await;

        self.insert_outbound(chat_id, body, None, None).await
    }

    /// Persist a reply. WA dispatch currently stubbed — TODO plumb mid mapping.
    #[tracing::instrument(skip(self, body))]
    pub async fn reply(
        &self,
        chat_id: Uuid,
        reply_to_message_id: Uuid,
        body: &str,
    ) -> Result<Message, AppError> {
        validate_body(body)?;
        self.find_chat(chat_id).await?;
        // TODO: fetch reply_to external_id + pass into WaCmd::SendText.
        self.insert_outbound(chat_id, body, Some(reply_to_message_id), None)
            .await
    }

    /// Persist a forwarded message. WA dispatch TODO.
    #[tracing::instrument(skip(self))]
    pub async fn forward(
        &self,
        src_chat_id: Uuid,
        src_mid: Uuid,
        dst_chat_id: Uuid,
    ) -> Result<Message, AppError> {
        let src_body: Option<(String,)> =
            sqlx::query_as("SELECT body FROM messages WHERE id = ?1 AND chat_id = ?2")
                .bind(src_mid.to_string())
                .bind(src_chat_id.to_string())
                .fetch_optional(&self.pool)
                .await?;
        let (body,) = src_body.ok_or(AppError::NotFound)?;
        self.insert_outbound(dst_chat_id, &body, None, Some(src_mid))
            .await
    }

    async fn find_chat(&self, chat_id: Uuid) -> Result<Chat, AppError> {
        let row: Option<(String, String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT id, wa_jid, display_name, last_message_at, created_at FROM chats WHERE id = ?1",
        )
        .bind(chat_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        let (id, jid, name, last, created) = row.ok_or(AppError::NotFound)?;
        Ok(Chat {
            id: Uuid::parse_str(&id).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
            wa_jid: jid,
            display_name: name,
            last_message_at: last
                .map(|s| s.parse())
                .transpose()
                .map_err(|e: chrono::ParseError| AppError::Internal(anyhow::anyhow!(e)))?,
            created_at: created
                .parse()
                .map_err(|e: chrono::ParseError| AppError::Internal(anyhow::anyhow!(e)))?,
        })
    }

    async fn insert_outbound(
        &self,
        chat_id: Uuid,
        body: &str,
        reply_to_message_id: Option<Uuid>,
        forwarded_from_message_id: Option<Uuid>,
    ) -> Result<Message, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO messages \
             (id, chat_id, external_id, direction, body, \
              reply_to_message_id, forwarded_from_message_id, ts) \
             VALUES (?1, ?2, NULL, 'out', ?3, ?4, ?5, ?6)",
        )
        .bind(id.to_string())
        .bind(chat_id.to_string())
        .bind(body)
        .bind(reply_to_message_id.map(|u| u.to_string()))
        .bind(forwarded_from_message_id.map(|u| u.to_string()))
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE chats SET last_message_at = ?1 WHERE id = ?2")
            .bind(now.to_rfc3339())
            .bind(chat_id.to_string())
            .execute(&self.pool)
            .await?;

        let msg = Message {
            id,
            chat_id,
            external_id: None,
            direction: Direction::Out,
            body: body.to_string(),
            reply_to_message_id,
            forwarded_from_message_id,
            ts: now,
        };
        self.bc.publish(Event::NewMessage {
            chat_id,
            message: msg.clone(),
        });
        Ok(msg)
    }
}

fn validate_body(body: &str) -> Result<(), AppError> {
    if body.is_empty() {
        return Err(AppError::Validation("body empty".into()));
    }
    if body.len() > MAX_BODY_BYTES {
        return Err(AppError::Validation(format!(
            "body > {MAX_BODY_BYTES} bytes"
        )));
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
fn row_to_message(
    r: (
        String,
        String,
        Option<String>,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
    ),
) -> Result<Message, AppError> {
    let (id, chat_id, ext, dir, body, rep, fwd, ts) = r;
    Ok(Message {
        id: Uuid::parse_str(&id).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
        chat_id: Uuid::parse_str(&chat_id)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
        external_id: ext,
        direction: Direction::from_db(&dir).map_err(AppError::Validation)?,
        body,
        reply_to_message_id: rep
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
        forwarded_from_message_id: fwd
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
        ts: ts
            .parse()
            .map_err(|e: chrono::ParseError| AppError::Internal(anyhow::anyhow!(e)))?,
    })
}

#[cfg(test)]
mod tests {
    use super::{validate_body, MAX_BODY_BYTES};

    #[test]
    fn rejects_empty_body() {
        assert!(validate_body("").is_err());
    }

    #[test]
    fn accepts_normal_body() {
        assert!(validate_body("hello").is_ok());
    }

    #[test]
    fn rejects_oversized_body() {
        let s = "a".repeat(MAX_BODY_BYTES + 1);
        assert!(validate_body(&s).is_err());
    }
}
