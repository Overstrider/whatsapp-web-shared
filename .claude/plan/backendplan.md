# backendplan

## meta
repo: backend
lang: rust
stack: rust-2024 + axum 0.8 + sqlx 0.8 + whatsapp-rust (git pin) + podman
mode: BOOTSTRAP
crate_name: whatsapp-web-shared-backend
toolchain: 1.84 stable (rust-toolchain.toml pinned)
bind: 0.0.0.0:8080

## Scope
Backend = sole WA talker. Holds one global WA session, persists state + session blob to Podman volumes. Serves REST + WS to many email-auth'd users via HttpOnly signed cookie. Orchestrates: magic-code email auth → cookie session → WA QR pairing → chat list / message stream / send+reply+forward text. v1 text-only. Restart-safe. Clean arch: domain/app/infra/routes. Single writer task owns WA client to serialize cmds.

## Module Layout

```
backend/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml              # channel = "1.84"
├── .sqlx/                           # sqlx prepare metadata (SQLX_OFFLINE=true in build)
├── .env.example
├── migrations/
│   └── 0001_init.sql
├── scripts/
│   └── smoke/
│       ├── 01_login.sh
│       ├── 02_qr.sh
│       ├── 03_send.sh
│       └── README.md
└── src/
    ├── main.rs                      # bootstrap: dotenvy → tracing → config → db → wa_client → router → serve + graceful shutdown
    ├── lib.rs                       # pub fn build_app(state) -> Router; re-exports for integration tests
    ├── config.rs                    # Env struct (Deserialize-free, hand-parsed std::env) + load() → anyhow::Result<Env>
    ├── error.rs                     # AppError (thiserror) + IntoResponse + ErrorBody JSON
    ├── state.rs                     # AppState { pool, auth_svc, wa_svc, chat_svc, broadcaster, cookie_key }
    ├── domain/
    │   ├── mod.rs
    │   ├── user.rs                  # User, Email newtype (E.164-less, but validated non-empty+@)
    │   ├── auth.rs                  # AuthCode, AuthSession, MagicCode newtype (6 digits)
    │   ├── wa.rs                    # WaConnState enum, QrCode newtype, GlobalSessionMeta
    │   ├── chat.rs                  # Chat, Jid newtype
    │   ├── message.rs               # Direction, Message
    │   └── events.rs                # Event enum (tagged serde) → WS frame source of truth
    ├── application/
    │   ├── mod.rs
    │   ├── auth_service.rs          # request_code, verify_code, current_user, logout, rate_limit
    │   ├── wa_session_service.rs    # start, state, qr, ensure_alive, reconnect; owns WaClient handle
    │   ├── chat_service.rs          # list, messages, new_chat, send, reply, forward; dedup on external_id
    │   └── realtime.rs              # Broadcaster (tokio::sync::broadcast<Event>, cap=1024)
    ├── infrastructure/
    │   ├── mod.rs
    │   ├── db.rs                    # SqlitePool init (WAL, foreign_keys, busy_timeout=5s, max_conn=8), sqlx::migrate!("./migrations")
    │   ├── repo_users.rs            # upsert_by_email, find_by_id
    │   ├── repo_auth.rs             # insert_code, find_valid_code, consume_code, insert_session, find_session_by_token, delete_session, purge_expired
    │   ├── repo_chats.rs            # upsert_by_jid, list_ordered, find_by_id, touch_last_msg
    │   ├── repo_messages.rs         # insert, list_page, find_by_external_id, find_by_id
    │   ├── repo_wa.rs               # load_singleton, save_state, save_qr, save_jid
    │   ├── email.rs                 # EmailSender trait + SmtpSender (lettre) + StdoutSender
    │   └── wa_client/
    │       ├── mod.rs               # WaClient trait + WaCmd enum + WaEvent enum + spawn(...) -> WaHandle
    │       ├── adapter.rs           # WaRustAdapter wraps whatsapp-rust; single owner task; mpsc<WaCmd> in, broadcast<WaEvent> out
    │       └── stub.rs              # StubAdapter (feature="wa_stub") for offline dev + smoke tests
    └── routes/
        ├── mod.rs                   # build_router(state) → Router; layer order: trace → cors → cookie → timeout
        ├── middleware.rs            # require_session extractor (FromRequestParts); returns User
        ├── auth.rs                  # /api/auth/login/{start,verify}, /api/auth/logout, /api/auth/session
        ├── wa.rs                    # /api/whatsapp/{state,qr,reconnect}
        ├── chats.rs                 # /api/chats*, /api/chats/:id/messages*, reply, forward
        ├── health.rs                # /api/health
        └── ws.rs                    # /ws upgrade; auth via cookie; broadcast::Receiver → ws::Message::Text(json)
```

## Dependency List (exact versions + why)

```toml
[package]
name = "whatsapp-web-shared-backend"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = { version = "0.8", features = ["ws", "macros", "tokio", "http1", "json", "query"] }
axum-extra = { version = "0.10", features = ["cookie-private", "typed-header"] }
tokio = { version = "1.47", features = ["full"] }
tokio-stream = { version = "0.1", features = ["sync"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors", "timeout", "limit"] }
sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio", "tls-rustls-ring", "sqlite", "uuid", "chrono", "migrate", "macros"] }
whatsapp-rust = { git = "https://github.com/oxidezap/whatsapp-rust", branch = "main" }   # specialist pins exact rev at scaffold
lettre = { version = "0.11", default-features = false, features = ["smtp-transport", "tokio1-rustls-tls", "builder", "hostname"] }
argon2 = "0.5"
phonenumber = "0.3"
uuid = { version = "1.11", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
dotenvy = "0.15"
rand = "0.8"
futures = "0.3"
hex = "0.4"
base64 = "0.22"

[dev-dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }
tokio = { version = "1.47", features = ["macros", "rt-multi-thread", "test-util"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "cookies"] }

[features]
default = []
wa_stub = []                         # swap WaRustAdapter → StubAdapter at wire-time

[profile.release]
lto = "thin"
codegen-units = 1
strip = "debuginfo"
```

Why each pick: axum 0.8 = latest matching arcplan; sqlx 0.8 rustls-ring = no openssl pain in slim-bookworm; whatsapp-rust git = only distribution; argon2 = defense-in-depth for short-lived code at rest; phonenumber = server-side double-check of frontend libphonenumber-js; tokio-stream sync = BroadcastStream for WS fan-out; rand = OsRng for tokens + 6-digit codes; hex = 32-byte token encoding; base64 = QR png encoding pass-through if adapter returns raw bytes.

## Domain Types (types + invariants)

```rust
// domain/user.rs
pub struct User { pub id: Uuid, pub email: String, pub created_at: DateTime<Utc> }
pub struct Email(String);            // constructor: trim + lowercase + must contain '@' + len <= 320

// domain/auth.rs
pub struct MagicCode(String);        // exactly 6 ASCII digits; Display redacted as "******"
pub struct AuthCode {
    pub id: Uuid, pub email: String, pub code_hash: String,   // argon2id
    pub expires_at: DateTime<Utc>, pub consumed: bool, pub created_at: DateTime<Utc>,
}
pub struct AuthSession {
    pub token: String,               // 32-byte OsRng → hex (64 chars)
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>, pub expires_at: DateTime<Utc>,
}

// domain/wa.rs
#[serde(tag="type", content="detail")]
pub enum WaConnState { Disconnected, WaitingQr, Connecting, Connected, LoggedOut, Error(String) }
pub struct QrCode(pub String);       // opaque; adapter decides format (raw ws qr string OR data:image/png;base64,...)
pub struct GlobalSessionMeta {
    pub state: WaConnState, pub jid: Option<String>,
    pub last_qr: Option<String>, pub last_connected_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

// domain/chat.rs
pub struct Jid(String);              // non-empty, must contain '@' (WA format user@s.whatsapp.net or group@g.us)
pub struct Chat {
    pub id: Uuid, pub wa_jid: String, pub display_name: String,
    pub last_message_at: Option<DateTime<Utc>>, pub created_at: DateTime<Utc>,
}

// domain/message.rs
#[serde(rename_all="lowercase")]
pub enum Direction { In, Out }
pub struct Message {
    pub id: Uuid, pub chat_id: Uuid,
    pub external_id: Option<String>,                         // WA mid; UNIQUE; dedup key
    pub direction: Direction, pub body: String,
    pub reply_to_message_id: Option<Uuid>,
    pub forwarded_from_message_id: Option<Uuid>,
    pub ts: DateTime<Utc>,
}

// domain/events.rs
#[serde(tag="type")]
pub enum Event {
    WaStateChange { state: WaConnState },
    QrUpdate { qr: String },
    NewMessage { chat_id: Uuid, message: Message },
    ChatUpserted { chat: Chat },
}
```

Invariants:
- `Email` / `Jid` / `MagicCode` newtypes — construct via `try_from(&str)`, panic-free.
- `Message.body` ≤ 65_535 bytes (enforce at route validator).
- `external_id` UNIQUE → dedup on WA inbound loop.
- `AuthCode.expires_at` = created + 5 min; `AuthSession.expires_at` = created + 30 d (env-configurable).

### Error type (error.rs)

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("db: {0}")]        Db(#[from] sqlx::Error),
    #[error("wa: {0}")]        Wa(String),
    #[error("auth: {0}")]      Auth(AuthReason),
    #[error("validation: {0}")] Validation(String),
    #[error("email: {0}")]     Email(String),
    #[error("not_found")]      NotFound,
    #[error("conflict: {0}")]  Conflict(String),
    #[error("rate_limited")]   RateLimited,
    #[error("internal: {0}")]  Internal(#[from] anyhow::Error),
}

pub enum AuthReason { InvalidCode, ExpiredCode, NoSession, SessionExpired }
```

`IntoResponse` mapping:
- Db → 500 `{error:{code:"db_error"}}` (log full).
- Wa → 502 `wa_unavailable`.
- Auth(InvalidCode|ExpiredCode) → 400 `invalid_code`.
- Auth(NoSession|SessionExpired) → 401 `unauthenticated`.
- Validation → 422 `validation_error` (with message).
- Email → 500 `email_failed` (log full).
- NotFound → 404.
- Conflict → 409.
- RateLimited → 429 `rate_limited`.
- Internal → 500 `internal`.

Never leak inner strings of Db/Internal variants to client; always tracing::error! full.

## Application Services (function signatures)

```rust
// application/auth_service.rs
pub struct AuthService { pool: SqlitePool, email: Arc<dyn EmailSender>, cfg: Arc<Env> }
impl AuthService {
    pub async fn request_code(&self, email: &str) -> Result<(), AppError>;
    pub async fn verify_code(&self, email: &str, code: &str) -> Result<AuthSession, AppError>;
    pub async fn current_user(&self, token: &str) -> Result<User, AppError>;
    pub async fn logout(&self, token: &str) -> Result<(), AppError>;
}
// internal: check rate-limit via auth_codes insert count per email per hour (max 5).

// application/wa_session_service.rs
pub struct WaSessionService { pool: SqlitePool, handle: WaHandle, bc: Broadcaster }
impl WaSessionService {
    pub async fn start(&self) -> Result<(), AppError>;           // spawn event-forwarding task on boot
    pub fn state(&self) -> WaConnState;                          // cached atomic; updated by event loop
    pub fn qr(&self) -> Option<String>;                          // last QR; None if Connected
    pub async fn ensure_alive(&self) -> Result<(), AppError>;    // noop if Connected, else trigger reconnect
    pub async fn reconnect(&self) -> Result<(), AppError>;
}

// application/chat_service.rs
pub struct ChatService { pool: SqlitePool, handle: WaHandle, bc: Broadcaster }
impl ChatService {
    pub async fn list(&self) -> Result<Vec<Chat>, AppError>;
    pub async fn messages(&self, chat_id: Uuid, before: Option<DateTime<Utc>>, limit: u32) -> Result<Vec<Message>, AppError>;
    pub async fn new_chat(&self, phone_e164: &str) -> Result<Chat, AppError>;           // validate via phonenumber, build jid
    pub async fn send(&self, chat_id: Uuid, body: &str) -> Result<Message, AppError>;
    pub async fn reply(&self, chat_id: Uuid, reply_to_mid: Uuid, body: &str) -> Result<Message, AppError>;
    pub async fn forward(&self, src_chat_id: Uuid, src_mid: Uuid, dst_chat_id: Uuid) -> Result<Message, AppError>;
}

// application/realtime.rs
pub struct Broadcaster { tx: broadcast::Sender<Event> }
impl Broadcaster {
    pub fn new(cap: usize) -> Self;                              // cap=1024
    pub fn subscribe(&self) -> broadcast::Receiver<Event>;
    pub fn publish(&self, e: Event);                             // lossy on Lagged → log warn, skip
}
```

Rate-limit: `request_code` counts rows in `auth_codes` where email=? AND created_at > now-1h → reject if >= 5 with `RateLimited`.

## Infrastructure

### db.rs
```rust
pub async fn init(database_url: &str) -> anyhow::Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePoolOptions::new().max_connections(8).connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
```
All queries use `sqlx::query!`/`query_as!` macros with `.sqlx` offline metadata. DateTime stored as RFC3339 TEXT.

### wa_client (adapter pattern)

```rust
// wa_client/mod.rs
pub enum WaCmd {
    SendText { jid: String, body: String, reply_to_external_id: Option<String>, ack: oneshot::Sender<Result<String, String>> },
    Reconnect { ack: oneshot::Sender<Result<(), String>> },
    GetState { ack: oneshot::Sender<WaConnState> },
}
pub enum WaEvent {
    StateChanged(WaConnState),
    QrUpdated(String),
    MessageReceived { external_id: String, jid: String, body: String, ts: DateTime<Utc>, from_me: bool, reply_to_external_id: Option<String> },
    ChatMetaUpdated { jid: String, display_name: String },
}
pub struct WaHandle {
    cmd_tx: mpsc::Sender<WaCmd>,
    evt_rx: broadcast::Receiver<WaEvent>,
    state:  Arc<ArcSwap<WaConnState>>,
}
pub trait WaClient: Send + Sync {
    fn spawn(session_dir: PathBuf) -> anyhow::Result<WaHandle>;
}
```
- adapter.rs = owns whatsapp-rust client, single tokio task pumps cmd rx + WA events; translates → WaEvent + updates `state` atomic.
- stub.rs = deterministic fake for smoke/integration; emits fake QR + echo on SendText.
- Session file: `{WA_SESSION_PATH}/session.bin`. Write-temp + rename on persist. Load error → clear + log warn → WaitingQr.
- Reconnect backoff: `delay_ms = min(30_000, 1000 * 2^attempt) + jitter(±20%)`.

### email.rs

```rust
#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_code(&self, to: &str, code: &str) -> Result<(), AppError>;
}
pub struct SmtpSender { tx: AsyncSmtpTransport<Tokio1Executor>, from: String }
pub struct StdoutSender;
```
Factory: `EMAIL_MODE=stdout` → StdoutSender (logs `CODE email=<> code=<>`); `smtp` → SmtpSender via `SmtpTransport::from_url(SMTP_URL)`-style build; SMTP_URL missing → fallback StdoutSender + warn.

### repo_* files
All functions `async fn(pool: &SqlitePool, ...) -> Result<T, sqlx::Error>`. UUIDs stored as TEXT (Display). Direction stored as TEXT "in"|"out".

## Routes (axum router wiring)

Router assembled in `routes/mod.rs`:
```
api = Router::new()
  .route("/auth/login/start",            post(auth::login_start))
  .route("/auth/login/verify",           post(auth::login_verify))
  .route("/auth/logout",                 post(auth::logout).layer(require_session))
  .route("/auth/session",                get(auth::session).layer(require_session))
  .route("/whatsapp/state",              get(wa::state).layer(require_session))
  .route("/whatsapp/qr",                 get(wa::qr).layer(require_session))
  .route("/whatsapp/reconnect",          post(wa::reconnect).layer(require_session))
  .route("/chats",                       get(chats::list).post(chats::new_chat).layer(require_session))
  .route("/chats/:id/messages",          get(chats::messages).post(chats::send).layer(require_session))
  .route("/chats/:id/messages/:mid/reply",   post(chats::reply).layer(require_session))
  .route("/chats/:id/messages/:mid/forward", post(chats::forward).layer(require_session))
  .route("/health",                      get(health::health));
app = Router::new().nest("/api", api).route("/ws", get(ws::upgrade))
  .layer(TraceLayer::new_for_http())
  .layer(CorsLayer::new().allow_origin([FRONTEND_ORIGIN]).allow_credentials(true)
         .allow_methods([GET, POST]).allow_headers([CONTENT_TYPE]))
  .layer(TimeoutLayer::new(Duration::from_secs(30)))
  .with_state(state);
```

### Route detail (method path → handler → auth → JSON in / out → status)

| Method | Path | Handler | Auth | In | Out | Status |
|---|---|---|---|---|---|---|
| POST | /api/auth/login/start | `login_start` | none | `{email}` | `-` | 204; 422 invalid email; 429 rate |
| POST | /api/auth/login/verify | `login_verify` | none | `{email, code}` | `{user}` + `Set-Cookie: sid=...` | 200; 400 invalid_code |
| POST | /api/auth/logout | `logout` | session | `-` | `-` | 204 |
| GET | /api/auth/session | `session` | session | `-` | `{user}` | 200; 401 |
| GET | /api/whatsapp/state | `state` | session | `-` | `{state: WaState}` | 200 |
| GET | /api/whatsapp/qr | `qr` | session | `-` | `{qr: string}` | 200; 409 if Connected |
| POST | /api/whatsapp/reconnect | `reconnect` | session | `-` | `-` | 202 |
| GET | /api/chats | `list` | session | `-` | `[Chat]` | 200 |
| POST | /api/chats | `new_chat` | session | `{phone}` | `Chat` | 201; 422 bad phone |
| GET | /api/chats/:id/messages?before=&limit= | `messages` | session | qs | `[Message]` | 200; 404 |
| POST | /api/chats/:id/messages | `send` | session | `{body}` | `Message` | 201; 502 wa_unavailable |
| POST | /api/chats/:id/messages/:mid/reply | `reply` | session | `{body}` | `Message` | 201 |
| POST | /api/chats/:id/messages/:mid/forward | `forward` | session | `{dst_chat_id}` | `Message` | 201 |
| GET | /api/health | `health` | none | `-` | `{ok:true, wa:WaState}` | 200 |
| GET | /ws | `upgrade` | session (cookie on upgrade req) | WS | WS(Event json) | 101; 4401 close if no session |

`limit` default=50, max=200. `before` = iso8601. Handler validation: reject `body` empty or > 65_535 bytes.

### require_session middleware
`FromRequestParts` extractor: reads private cookie `sid` via axum-extra `PrivateCookieJar`. Looks up `auth_sessions` row; if expired or missing → `AppError::Auth(NoSession)`. Inserts `User` into request extensions. Handlers extract via `Extension<User>`.

### /ws upgrade
1. Extract cookie `sid`; if missing/invalid → return `ws.close(4401)` after handshake (browser WebSocket cannot read cookies on upgrade reply; axum closes frame).
2. On open: `rx = broadcaster.subscribe()`.
3. Spawn: forward `BroadcastStream(rx)` → `ws.send(Message::Text(serde_json::to_string(&event)))`.
4. Ignore inbound client frames v1 (log trace). Pong on ping.
5. On `Lagged(n)` → send `{"type":"WsLag","dropped":n}` non-tagged frame → continue.
6. Events are NOT filtered by user v1 — global session = shared view; all authed users get all events.

## Database Migrations (0001_init.sql — full SQL)

```sql
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
```

Decision: singleton-column pattern via `CHECK (id=1)` + seed row. No separate `wa_connection_state` table — state lives inline in `wa_global_session`. WAL enabled at runtime in `db.rs` (not via PRAGMA in SQL migration — SQLite persists journal_mode across connections once set).

## Containerfile + Podman

### Containerfile (repo root)
```
FROM rust:1.84-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/rust-toolchain.toml ./
COPY backend/src ./src
COPY backend/migrations ./migrations
COPY backend/.sqlx ./.sqlx
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl tini \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -u 10001 -m -s /usr/sbin/nologin app \
    && mkdir -p /data /wa-session && chown -R 10001:10001 /data /wa-session
COPY --from=builder /app/target/release/whatsapp-web-shared-backend /usr/local/bin/app
COPY backend/migrations /migrations
USER 10001
ENV DATABASE_URL=sqlite:/data/app.db \
    WA_SESSION_PATH=/wa-session \
    BIND_ADDR=0.0.0.0:8080 \
    RUST_LOG=info,whatsapp_web_shared_backend=debug
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD curl -fsS http://localhost:8080/api/health || exit 1
ENTRYPOINT ["/usr/bin/tini","--"]
CMD ["/usr/local/bin/app"]
```

### podman-commands.sh (repo root, copy-paste ready)
```bash
#!/usr/bin/env bash
set -euo pipefail
IMAGE="whatsapp-web-shared:latest"
NAME="wa-backend"

cmd_build()   { podman build -t "$IMAGE" -f Containerfile .; }
cmd_volumes() { podman volume create wa-data 2>/dev/null || true; podman volume create wa-session 2>/dev/null || true; }
cmd_run()     { cmd_volumes
                podman run -d --name "$NAME" -p 8080:8080 \
                  -v wa-data:/data -v wa-session:/wa-session \
                  --env-file .env --restart unless-stopped "$IMAGE"; }
cmd_stop()    { podman stop "$NAME" || true; podman rm "$NAME" || true; }
cmd_restart() { podman restart "$NAME"; }
cmd_logs()    { podman logs -f "$NAME"; }
cmd_shell()   { podman exec -it "$NAME" /bin/bash; }

case "${1:-}" in
  build|volumes|run|stop|restart|logs|shell) "cmd_$1" ;;
  *) echo "usage: $0 {build|volumes|run|stop|restart|logs|shell}"; exit 1 ;;
esac
```

## .env.example

```
# Binding
BIND_ADDR=0.0.0.0:8080

# Database (SQLite on named volume)
DATABASE_URL=sqlite:/data/app.db

# WhatsApp session persistence
WA_SESSION_PATH=/wa-session

# Signed cookie master key — 64 hex chars (generate: openssl rand -hex 32)
COOKIE_KEY=CHANGE_ME_64_HEX_CHARS_0000000000000000000000000000000000000000000000000000

# Session + code TTL
SESSION_TTL_DAYS=30
AUTH_CODE_TTL_SECS=300
RATE_LIMIT_CODES_PER_HOUR=5

# CORS origin of frontend
FRONTEND_ORIGIN=http://localhost:3000

# Email sender: stdout | smtp
EMAIL_MODE=stdout
SMTP_HOST=
SMTP_PORT=587
SMTP_USER=
SMTP_PASS=
SMTP_FROM=noreply@example.com

# Logging
RUST_LOG=info,whatsapp_web_shared_backend=debug

# Secure cookie flag: 1 in prod (https), 0 in dev
COOKIE_SECURE=0
```

## Testing Approach

- **Unit tests** (per-module `#[cfg(test)] mod tests`):
  - `domain::auth::MagicCode` → 6-digit invariant, parse round-trip.
  - `domain::chat::Jid` → accept/reject cases.
  - `application::auth_service` → hash+verify, rate-limit window, expired code.
  - `application::chat_service` → dedup on external_id, new_chat phone validation.

- **Integration tests** (`tests/*.rs`):
  - `tests/auth_flow.rs` → spin in-mem sqlite (`sqlite::memory:` + migrate), StubAdapter, hit routes via `tower::ServiceExt::oneshot` on Router. Assert cookie set + /session returns user.
  - `tests/ws_flow.rs` → publish fake event → subscriber receives it.
  - `tests/chats_flow.rs` → new_chat + send → StubAdapter echoes → NewMessage event + DB row.

- **Smoke scripts** (`backend/scripts/smoke/`):
  - `01_login.sh` → curl login/start; grep stdout log for code; curl login/verify; save cookie jar.
  - `02_qr.sh` → curl /api/whatsapp/state; curl /api/whatsapp/qr; pipe to terminal QR renderer.
  - `03_send.sh` → curl POST /api/chats then /messages.
  - `README.md` → ordered run guide.

Offline build: `cargo sqlx prepare -- --lib` committed `.sqlx/` keeps `SQLX_OFFLINE=true` builder green.

## Implementation Order (topo-sorted for task-architect)

1. **Scaffold** — `backend/Cargo.toml` + `rust-toolchain.toml` + `src/main.rs` stub that binds and logs "hello". Verify `cargo run` serves 404.
2. **Config + error + db** — `config.rs` loads env; `error.rs` defines AppError + IntoResponse; `infrastructure/db.rs` opens SqlitePool WAL + runs `migrations/0001_init.sql`.
3. **Domain types** — all `src/domain/*` modules compile; zero IO, serde derive, newtype validators.
4. **Auth service + repos** — `repo_users`, `repo_auth`, `application/auth_service` (request_code/verify_code/current_user/logout + rate-limit). Unit tests green.
5. **Email sender** — `infrastructure/email.rs` trait + StdoutSender + SmtpSender factory. Wire into AuthService via `Arc<dyn EmailSender>`.
6. **Auth routes + cookie middleware** — `routes/auth.rs` + `routes/middleware.rs`; PrivateCookieJar with COOKIE_KEY; integration test for full magic-code flow.
7. **WA client trait + stub adapter** — `infrastructure/wa_client/{mod,stub}.rs`; spawn + WaHandle + cmd/event enums. Unit test cmd round-trip.
8. **WA session service + routes** — `application/wa_session_service` + `routes/wa.rs`; state/qr/reconnect endpoints backed by StubAdapter + Broadcaster.
9. **Realtime broadcaster** — `application/realtime.rs` + /ws upgrade in `routes/ws.rs`. Integration test: subscribe → publish → receive frame.
10. **Chat service + repos + routes** — `repo_chats`, `repo_messages`, `application/chat_service`, `routes/chats.rs` (list, new_chat, messages, send, reply, forward). Stub adapter echoes outbound → NewMessage event + DB upsert.
11. **Real whatsapp-rust adapter** — `infrastructure/wa_client/adapter.rs` wraps real lib; session blob load/save atomic; event-loop translates to WaEvent; plug in behind `default` features (swap stub via `wa_stub` feature for tests).
12. **Container + podman** — `Containerfile` multi-stage + `podman-commands.sh` at repo root; verify build + run + volume persistence across restart.
13. **Docs** — `backend/README.md` (build/run/smoke), repo-root `.env.example`.
14. **Smoke scripts** — `backend/scripts/smoke/*.sh` end-to-end manual test.

## Risks / Traps (caveman)

- whatsapp-rust API churn → adapter trait + `wa_stub` feature → swap to StubAdapter when upstream breaks → unblock non-WA work.
- sqlite WAL + concurrent WS-driven writes → max_conn=8 + `busy_timeout=5s` + inbound WA events serialized through WaClient task (single writer to chats/messages from WA side) → no contention.
- cookie key rotation → COOKIE_KEY change invalidates all sessions → document in README + warn at boot if key length != 64 hex.
- magic code brute force → 5/hr rate limit per email + consumed flag + 5-min TTL + argon2 hash → brute-infeasible.
- WA session file corruption → write to `session.bin.tmp` + fsync + rename → atomic; on load failure clear + warn + WaitingQr.
- broadcast lag → cap=1024; on Lagged send one `WsLag` frame + continue → client re-fetches via REST.
- CORS + cookie → `allow_credentials=true` requires exact origin (not `*`) → FRONTEND_ORIGIN env mandatory + boot-fail if empty in prod (COOKIE_SECURE=1).
- phone validation mismatch → backend `phonenumber::parse` as authoritative; frontend libphonenumber-js pre-check cosmetic.
- SQLite datetime → always store RFC3339 UTC via `chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)`; never rely on `CURRENT_TIMESTAMP` (different format).
- graceful shutdown → `tokio::signal::ctrl_c` → close broadcaster + WaClient handle + pool → prevent mid-write corruption.
- /ws cookie on upgrade → browsers auto-send; but Origin header required for CORS; validate Origin matches FRONTEND_ORIGIN in upgrade handler → reject otherwise 403.

PLAN_COMPLETE: backendplan.md
