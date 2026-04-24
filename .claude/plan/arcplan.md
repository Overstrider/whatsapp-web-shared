# arcplan

## meta
name: whatsapp-web-shared
repos: backend, frontend
mode: BOOTSTRAP

## Project Description
Full-stack shared WhatsApp Web clone. Rust+Axum backend = sole WA talker, holds one global WA session, serves text chats + realtime to many email-auth'd users via HttpOnly-cookie server sessions. Next.js+TS frontend renders WA-Web UI, consumes only backend API. SQLite file + WA session blob persist in Podman volumes → restart-safe. v1 text-only.

## Stack & Dependencies

### backend (Rust 2024, edition = "2024")
- axum 0.8 — HTTP + WS router
- tokio 1.47 (features = full) — async runtime
- tower 0.5 / tower-http 0.6 (trace, cors, cookie, timeout)
- axum-extra 0.10 (cookie-private feature) — signed HttpOnly cookies
- sqlx 0.8 (runtime-tokio, sqlite, migrate, uuid, chrono, macros) — DB + migrations
- whatsapp-rust → git pin `github.com/oxidezap/whatsapp-rust` rev = latest main SHA at scaffold (specialist pins exact)
- uuid 1.11 (v4, serde)
- chrono 0.4 (serde) — timestamps
- serde 1 / serde_json 1
- thiserror 2 — domain errors
- tracing 0.1 + tracing-subscriber 0.3 (env-filter, fmt)
- lettre 0.11 (smtp-transport, tokio1-rustls-tls, builder) — email send; dev fallback = stdout logger behind trait
- rand 0.8 — magic code gen
- argon2 0.5 — hash auth_codes at rest (defense-in-depth; code itself short-lived)
- phonenumber 0.3 — E.164 validation
- dotenvy 0.15 — env load
- figment 0.10 OR plain std::env — config (pick std::env + dotenvy, simpler)
- anyhow 1 — app-level bubbling in main only
- tokio-stream 0.1 — broadcast → WS

Realtime = WebSocket (not SSE). Reason: bidirectional room for future typing/read-receipts, axum ws built-in, single connection handles all event kinds.

### frontend (Next.js + TS)
- next 15.2 (App Router, React Server Components off for chat pages — client-heavy)
- react 19.0 / react-dom 19.0
- typescript 5.7 (strict)
- tailwindcss 4.0 — pick tailwind over CSS modules. Reason: WA-Web layout = dense utility classes, theming fast, smaller dev churn.
- zustand 5 — global client state (current chat, ws status, qr). Lighter than context+reducer, no provider hell.
- emoji-picker-react 4.12 — picker in composer
- clsx 2 — conditional classes
- zod 3.23 — runtime validation of API responses
- ky 1.7 — typed fetch wrapper (cookie credentials: 'include')
- native WebSocket (no socket.io) — matches backend axum ws, zero deps

### runtime
- Podman Containerfile, multi-stage: builder = rust:1.84-slim-bookworm → runtime = debian:bookworm-slim (ca-certs, non-root uid 10001)
- volumes: `wa-data:/data` (sqlite app.db + WAL), `wa-session:/wa-session` (whatsapp-rust store blob)
- healthcheck: curl -f localhost:8080/api/health
- frontend NOT containerized v1 → runs `next start` on host or Vercel; backend CORS allows FRONTEND_ORIGIN env

## Directory Structure

```
whatsapp-web-shared/
├── FEATURE.md
├── README.md
├── .env.example
├── Containerfile
├── podman-commands.sh
├── .claude/...
├── backend/
│   ├── Cargo.toml
│   ├── rust-toolchain.toml           # channel = "1.84"
│   ├── .sqlx/                        # offline query metadata
│   ├── migrations/
│   │   └── 0001_init.sql
│   └── src/
│       ├── main.rs                   # bootstrap: config → db → wa → router → serve
│       ├── lib.rs                    # re-export app_router for tests
│       ├── config.rs                 # Env struct + load()
│       ├── error.rs                  # AppError + IntoResponse
│       ├── domain/
│       │   ├── mod.rs
│       │   ├── user.rs               # User { id, email, created_at }
│       │   ├── auth.rs               # AuthCode, AuthSession, Email newtype
│       │   ├── wa.rs                 # WaConnState enum, QrCode, GlobalSessionMeta
│       │   ├── chat.rs               # Chat { id, wa_jid, display_name, last_ts }
│       │   └── message.rs            # Message { id, chat_id, external_id, direction, body, reply_to, forwarded_from, ts }
│       ├── application/
│       │   ├── mod.rs
│       │   ├── auth_service.rs       # request_code, verify_code, current_user
│       │   ├── wa_session_service.rs # qr, state, ensure_alive, reconnect, start
│       │   ├── chat_service.rs       # list, messages, new_chat, send, reply, forward
│       │   └── realtime.rs           # Broadcaster wrapping tokio::sync::broadcast<Event>
│       ├── infrastructure/
│       │   ├── mod.rs
│       │   ├── db.rs                 # SqlitePool init (WAL, foreign_keys, max_conn=8)
│       │   ├── repo_users.rs
│       │   ├── repo_auth.rs          # auth_codes + auth_sessions
│       │   ├── repo_chats.rs
│       │   ├── repo_messages.rs
│       │   ├── repo_wa.rs            # singleton row r/w
│       │   ├── email.rs              # EmailSender trait + SmtpSender + StdoutSender
│       │   └── wa_client.rs          # WaClient adapter wraps whatsapp-rust; spawns event loop → Broadcaster
│       └── routes/
│           ├── mod.rs                # build_router()
│           ├── auth.rs               # /api/auth/*
│           ├── wa.rs                 # /api/whatsapp/*
│           ├── chats.rs              # /api/chats + /api/chats/:id/*
│           ├── health.rs
│           ├── ws.rs                 # /ws upgrade + fan-out
│           └── middleware.rs         # require_session extractor
└── frontend/
    ├── package.json
    ├── tsconfig.json
    ├── next.config.ts
    ├── tailwind.config.ts
    ├── postcss.config.mjs
    ├── .env.example                  # NEXT_PUBLIC_API_BASE, NEXT_PUBLIC_WS_URL
    ├── public/
    └── src/
        ├── app/
        │   ├── layout.tsx
        │   ├── page.tsx              # redirect → /login or /chats
        │   ├── login/page.tsx
        │   ├── verify/page.tsx
        │   ├── qr/page.tsx
        │   └── chats/
        │       ├── layout.tsx        # sidebar + outlet
        │       ├── page.tsx          # empty state
        │       └── [chatId]/page.tsx
        ├── components/
        │   ├── Sidebar.tsx
        │   ├── ChatList.tsx
        │   ├── ChatListItem.tsx
        │   ├── ChatView.tsx
        │   ├── MessageBubble.tsx
        │   ├── Composer.tsx
        │   ├── EmojiPicker.tsx
        │   ├── QrPanel.tsx
        │   ├── WaStatePill.tsx
        │   └── NewChatDialog.tsx
        ├── lib/
        │   ├── api.ts                # ky instance, typed endpoints
        │   ├── ws.ts                 # WS client + event dispatch → zustand
        │   ├── schemas.ts            # zod types for API
        │   └── phone.ts              # E.164 client validation
        └── store/
            ├── session.ts            # user + wa state
            ├── chats.ts              # chat list + messages map
            └── ws.ts                 # connection status
```

## Module Guide

### backend
- `config.rs` — load env once, fail-fast on missing.
- `error.rs` — AppError enum → HTTP status + JSON body.
- `domain/*` — plain types + invariants, zero IO.
- `application/*` — orchestrate repos + wa_client + broadcaster; return domain types.
- `infrastructure/db.rs` — SqlitePool, WAL pragma, migrations run on boot.
- `infrastructure/repo_*` — sqlx query functions, one file per aggregate.
- `infrastructure/wa_client.rs` — adapter; hides whatsapp-rust API; exposes `connect()`, `qr_stream()`, `send_text(jid, body, reply_to)`, `event_stream()`.
- `infrastructure/email.rs` — trait EmailSender; two impls.
- `routes/*` — thin handlers; extract services via axum State<AppState>; validate input, call application, map to JSON.
- `routes/ws.rs` — upgrade → subscribe broadcaster → forward events filtered by user session.

### frontend
- `lib/api.ts` — single ky client, `credentials: 'include'`, 401 → redirect /login.
- `lib/ws.ts` — singleton WebSocket, reconnect w/ backoff, dispatches into zustand.
- `store/chats.ts` — normalized chat dict + message arrays keyed by chatId.
- `components/ChatView.tsx` — virtualized scroll optional v2; v1 plain list bottom-anchored.
- `components/Composer.tsx` — textarea + emoji-picker-react + send btn; supports reply-to quoted preview.

## Backend module breakdown (repo:backend)

### Scaffold
- Single crate `whatsapp-web-shared-backend`, edition "2024".
- `cargo run` binds `0.0.0.0:8080`.
- rust-toolchain.toml pins 1.84 stable (first stable supporting edition 2024 broadly).

### Domain types (shape)
```
enum WaConnState { Disconnected, WaitingQr, Connecting, Connected, LoggedOut, Error(String) }
struct User { id: Uuid, email: String, created_at: DateTime<Utc> }
struct AuthCode { id: Uuid, email: String, code_hash: String, expires_at: DateTime<Utc>, consumed: bool }
struct AuthSession { token: String /*opaque 32B hex*/, user_id: Uuid, expires_at: DateTime<Utc> }
struct Chat { id: Uuid, wa_jid: String, display_name: String, last_message_at: Option<DateTime<Utc>> }
enum Direction { In, Out }
struct Message { id: Uuid, chat_id: Uuid, external_id: Option<String>, direction: Direction, body: String, reply_to_message_id: Option<Uuid>, forwarded_from_message_id: Option<Uuid>, ts: DateTime<Utc> }
```

### Application services (signatures)
```
AuthService::request_code(email) -> Result<()>
AuthService::verify_code(email, code) -> Result<AuthSession>
AuthService::current_user(token) -> Result<User>
WaSessionService::state() -> WaConnState
WaSessionService::qr() -> Option<String>   // data URL png OR raw string
WaSessionService::reconnect() -> Result<()>
ChatService::list() -> Vec<Chat>
ChatService::messages(chat_id, limit, before_ts) -> Vec<Message>
ChatService::new_chat(phone_e164) -> Chat
ChatService::send(chat_id, body) -> Message
ChatService::reply(chat_id, reply_to_mid, body) -> Message
ChatService::forward(src_chat_id, src_mid, dst_chat_id) -> Message
Broadcaster::subscribe() -> broadcast::Receiver<Event>
```

### Realtime event enum (serialized as tagged JSON)
```
#[serde(tag="type")]
enum Event {
  WaStateChange { state: WaConnState },
  QrUpdate { qr: String },
  NewMessage { chat_id: Uuid, message: Message },
  ChatUpserted { chat: Chat },
}
```

### Routes (exact)
```
POST /api/auth/login/start              { email } -> 204
POST /api/auth/login/verify             { email, code } -> Set-Cookie + { user }
POST /api/auth/logout                   -> 204
GET  /api/auth/session                  -> { user } | 401
GET  /api/whatsapp/state                -> { state }
GET  /api/whatsapp/qr                   -> { qr } | 409 if Connected
POST /api/whatsapp/reconnect            -> 202
GET  /api/chats                         -> [Chat]
GET  /api/chats/:id/messages?before&limit -> [Message]
POST /api/chats                         { phone } -> Chat
POST /api/chats/:id/messages            { body } -> Message
POST /api/chats/:id/messages/:mid/reply { body } -> Message
POST /api/chats/:id/messages/:mid/forward { dst_chat_id } -> Message
GET  /api/health                        -> { ok: true, wa: state }
GET  /ws                                -> websocket upgrade
```

All /api/* except `/api/auth/login/*`, `/api/health` require cookie session middleware.

### Auth flow
1. POST login/start → generate 6-digit code → argon2 hash → insert auth_codes (5 min TTL) → EmailSender.send(code).
2. POST login/verify → lookup unexpired unconsumed code by email, verify hash, mark consumed, upsert user by email, create auth_session (token = 32B rand hex, 30 day TTL), set cookie `sid=<token>; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=2592000`.
3. Middleware reads `sid`, joins auth_sessions → users, injects User into req ext.

### WA client lifecycle
- On startup: `WaClient::connect()` loads session blob from `WA_SESSION_PATH`. If missing → state = WaitingQr, stream QR events → broadcaster. If present → state = Connecting → Connected.
- Event loop (tokio task) consumes whatsapp-rust events → maps to domain Event → broadcaster.send + persist (chats/messages upsert).
- Reconnect: exponential backoff 1s,2s,4s,8s,16s,30s cap; reset on success.

## Frontend module breakdown (repo:frontend)

### Routes
- `/login` — email input → POST /api/auth/login/start → push `/verify?email=`.
- `/verify` — code input → POST /api/auth/login/verify → if wa state != Connected → `/qr`, else `/chats`.
- `/qr` — poll GET /api/whatsapp/state + WS `QrUpdate`; render QR; auto-redirect `/chats` on Connected.
- `/chats` — sidebar (ChatList) + empty outlet.
- `/chats/[chatId]` — ChatView + Composer.

### State (zustand)
- `session`: { user, waState }
- `chats`: { byId: Record<string,Chat>, order: string[], messagesByChat: Record<string,Message[]> }
- `ws`: { status: 'connecting'|'open'|'closed', reconnectAt }

### WS client
- connect on auth; url = `NEXT_PUBLIC_WS_URL` (ws://host/ws); credentials via cookie auto.
- on message: zod-parse Event → dispatch: NewMessage → append + bump chat; QrUpdate → session.setQr; WaStateChange → session.setWaState; ChatUpserted → chats.upsert.
- reconnect: 1s,2s,4s,8s cap 30s.

### Phone validation
- `lib/phone.ts` → libphonenumber-js 1.11 — parse E.164, reject invalid → inline error in NewChatDialog.

## Podman setup

### Containerfile (outline)
```
FROM rust:1.84-slim-bookworm AS builder
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations
COPY backend/.sqlx ./.sqlx
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -u 10001 -m app
COPY --from=builder /app/target/release/whatsapp-web-shared-backend /usr/local/bin/app
COPY backend/migrations /migrations
USER app
ENV DATABASE_URL=sqlite:/data/app.db
ENV WA_SESSION_PATH=/wa-session
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD curl -f http://localhost:8080/api/health || exit 1
CMD ["/usr/local/bin/app"]
```

### Volumes + commands (`podman-commands.sh`)
```
podman volume create wa-data
podman volume create wa-session
podman build -t whatsapp-web-shared:latest -f Containerfile .
podman run -d --name wa-backend \
  -p 8080:8080 \
  -v wa-data:/data \
  -v wa-session:/wa-session \
  --env-file .env \
  whatsapp-web-shared:latest
podman stop wa-backend
podman start wa-backend
podman restart wa-backend
podman logs -f wa-backend
```

### .env.example
```
DATABASE_URL=sqlite:/data/app.db
WA_SESSION_PATH=/wa-session
BIND_ADDR=0.0.0.0:8080
COOKIE_KEY=<64-hex chars>           # axum-extra private cookie master
SESSION_TTL_DAYS=30
AUTH_CODE_TTL_SECS=300
FRONTEND_ORIGIN=http://localhost:3000
EMAIL_MODE=stdout                   # stdout | smtp
SMTP_HOST=
SMTP_PORT=587
SMTP_USER=
SMTP_PASS=
SMTP_FROM=noreply@example.com
RUST_LOG=info,whatsapp_web_shared_backend=debug
```

## Database schema (migrations/0001_init.sql)

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE users (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL
);

CREATE TABLE auth_codes (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL,
  code_hash TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  consumed INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_auth_codes_email_expires ON auth_codes(email, expires_at);

CREATE TABLE auth_sessions (
  token TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL
);
CREATE INDEX idx_auth_sessions_user ON auth_sessions(user_id);

CREATE TABLE wa_global_session (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  state TEXT NOT NULL,                 -- WaConnState stringified
  jid TEXT,
  last_qr TEXT,
  last_connected_at TEXT,
  updated_at TEXT NOT NULL
);
INSERT INTO wa_global_session (id, state, updated_at) VALUES (1, 'Disconnected', CURRENT_TIMESTAMP);

CREATE TABLE chats (
  id TEXT PRIMARY KEY,
  wa_jid TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  last_message_at TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_chats_last_msg ON chats(last_message_at DESC);

CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  chat_id TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
  external_id TEXT UNIQUE,             -- WA message id for dedup
  direction TEXT NOT NULL,             -- 'in' | 'out'
  body TEXT NOT NULL,
  reply_to_message_id TEXT REFERENCES messages(id) ON DELETE SET NULL,
  forwarded_from_message_id TEXT REFERENCES messages(id) ON DELETE SET NULL,
  ts TEXT NOT NULL
);
CREATE INDEX idx_messages_chat_ts ON messages(chat_id, ts DESC);
CREATE INDEX idx_messages_external ON messages(external_id);
```

WAL mode enabled at runtime via `PRAGMA journal_mode=WAL` in db.rs init.

## cross-repo contracts

### HTTP contracts (JSON)
```
User        { id: uuid, email: string, created_at: iso8601 }
Chat        { id: uuid, wa_jid: string, display_name: string, last_message_at: iso8601|null }
Message     { id: uuid, chat_id: uuid, external_id: string|null, direction: "in"|"out",
              body: string, reply_to_message_id: uuid|null,
              forwarded_from_message_id: uuid|null, ts: iso8601 }
WaState     "Disconnected"|"WaitingQr"|"Connecting"|"Connected"|"LoggedOut"|{"Error": string}
ErrorBody   { error: { code: string, message: string } }
```

### WS event frames (JSON, tagged by `type`)
```
{ "type":"WaStateChange", "state": <WaState> }
{ "type":"QrUpdate", "qr": "<string>" }
{ "type":"NewMessage", "chat_id":"<uuid>", "message": <Message> }
{ "type":"ChatUpserted", "chat": <Chat> }
```

### Cookie
Name `sid`, HttpOnly, Secure (prod), SameSite=Lax, Path=/, signed via axum-extra private key. Frontend sends via `credentials: 'include'`.

### CORS
Backend allows `FRONTEND_ORIGIN` only, `allow_credentials=true`, methods GET/POST, headers content-type.

## Implementation Strategy (ordered)
1. backend: cargo init, deps pinned, rust-toolchain, .sqlx dir scaffold.
2. backend: config + error + domain types + migrations + db init (WAL).
3. backend: repo_users/auth + AuthService + stdout EmailSender + /api/auth routes + cookie middleware.
4. backend: wa_client adapter stub (returns fake QR + fake events) + WaSessionService + /api/whatsapp routes + broadcaster.
5. backend: /ws route (upgrade, auth via cookie, subscribe broadcaster).
6. backend: repo_chats/messages + ChatService + /api/chats routes.
7. backend: replace wa_client stub → real whatsapp-rust wrapper; wire event loop → persist + broadcast.
8. backend: health, CORS, tracing, graceful shutdown.
9. frontend: next scaffold, tailwind, zod, ky, zustand.
10. frontend: /login + /verify + session bootstrap (GET /api/auth/session).
11. frontend: /qr (poll + WS).
12. frontend: /chats sidebar + ChatList + NewChatDialog.
13. frontend: /chats/[chatId] ChatView + Composer + EmojiPicker.
14. frontend: WS client + store wiring + reconnect.
15. Containerfile + podman-commands.sh + README.
16. E2E smoke: build image, run container, login, scan QR, send text, receive text, restart container, verify persistence.

## Execution Order
backend → frontend. Serial. Frontend consumes backend contracts; parallelism yields no gain in v1 (schema here is authoritative — frontend can mock against it if needed, but spec has 1 dev).

## Risks and Edge Cases
- whatsapp-rust API churn → abstract behind `WaClient` trait; pin git rev; stub impl for tests.
- global session concurrent writers → single WaClient task owns send queue; services `mpsc::send` requests, await oneshot reply.
- reconnect storms → exponential backoff + jitter ±20%.
- email deliverability → dev stdout, prod SMTP via lettre; log msg id.
- SQLite writer contention → WAL + max_connections=8 + `busy_timeout=5000`.
- auth code brute force → rate-limit login/start per email (5/hour) + login/verify per email (5/code).
- cookie CSRF → SameSite=Lax blocks cross-site POST; no form-encoded endpoints; JSON only.
- clock skew → all TTL compared via server `Utc::now()`; never trust client.
- phone validation → libphonenumber on frontend + phonenumber crate on backend (double-check).
- session blob corruption → on deserialize error, clear + force new QR + log warn.
- WS auth → cookie sent on upgrade (browsers do); on missing session → close 4401.
- Podman volume permissions → app runs uid 10001; chown handled via named volume default or init entrypoint `mkdir -p /data /wa-session`.

## Out of scope (v1)
Media, voice, video, docs, stickers, reactions, typing indicators, read receipts, history sync, contact import, backup, multi-device WA, per-user WA sessions, push notifications, i18n, dark-mode toggle (default to WA-Web light).

ARCPLAN_COMPLETE
