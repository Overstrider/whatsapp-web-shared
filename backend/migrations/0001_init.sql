-- 0001_init.sql — full schema per backendplan.md section "Database Migrations".
-- Decisions:
--   * UUIDs stored as TEXT (canonical Display form, lowercase-hyphenated).
--   * Timestamps stored as RFC3339 UTC TEXT (produced by chrono at app layer).
--   * wa_global_session uses singleton-column pattern (CHECK id=1) with one seed row.
--   * WAL / synchronous / foreign_keys set at runtime in infrastructure/db.rs
--     (pragmas set via PRAGMA in SQL don't persist reliably across all sqlx paths).

PRAGMA foreign_keys = ON;

CREATE TABLE users (
  id           TEXT PRIMARY KEY,
  email        TEXT NOT NULL UNIQUE,
  created_at   TEXT NOT NULL
);

CREATE TABLE auth_codes (
  id           TEXT PRIMARY KEY,
  email        TEXT NOT NULL,
  code_hash    TEXT NOT NULL,
  expires_at   TEXT NOT NULL,
  consumed     INTEGER NOT NULL DEFAULT 0,
  created_at   TEXT NOT NULL
);
CREATE INDEX idx_auth_codes_email_expires ON auth_codes(email, expires_at);
CREATE INDEX idx_auth_codes_email_created ON auth_codes(email, created_at);

CREATE TABLE auth_sessions (
  token        TEXT PRIMARY KEY,
  user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at   TEXT NOT NULL,
  expires_at   TEXT NOT NULL
);
CREATE INDEX idx_auth_sessions_user ON auth_sessions(user_id);
CREATE INDEX idx_auth_sessions_expires ON auth_sessions(expires_at);

CREATE TABLE wa_global_session (
  id                  INTEGER PRIMARY KEY CHECK (id = 1),
  state               TEXT NOT NULL,
  jid                 TEXT,
  last_qr             TEXT,
  last_connected_at   TEXT,
  updated_at          TEXT NOT NULL
);
INSERT INTO wa_global_session (id, state, updated_at)
VALUES (1, 'Disconnected', strftime('%Y-%m-%dT%H:%M:%fZ','now'));

CREATE TABLE chats (
  id                TEXT PRIMARY KEY,
  wa_jid            TEXT NOT NULL UNIQUE,
  display_name      TEXT NOT NULL,
  last_message_at   TEXT,
  created_at        TEXT NOT NULL
);
CREATE INDEX idx_chats_last_msg ON chats(last_message_at DESC);

CREATE TABLE messages (
  id                          TEXT PRIMARY KEY,
  chat_id                     TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
  external_id                 TEXT UNIQUE,
  direction                   TEXT NOT NULL CHECK (direction IN ('in','out')),
  body                        TEXT NOT NULL,
  reply_to_message_id         TEXT REFERENCES messages(id) ON DELETE SET NULL,
  forwarded_from_message_id   TEXT REFERENCES messages(id) ON DELETE SET NULL,
  ts                          TEXT NOT NULL
);
CREATE INDEX idx_messages_chat_ts ON messages(chat_id, ts DESC);
CREATE INDEX idx_messages_external ON messages(external_id);
