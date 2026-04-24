# whatsapp-web-shared-backend

Single-crate Rust 2021 / Axum 0.8 backend for the shared WhatsApp-Web clone.
Holds one global WA session, persists state + session blob to Podman volumes,
serves REST + WS to many email-authenticated users via HttpOnly signed cookies.

## MVP simplifications (read before you dig in)

This scaffold intentionally keeps a handful of pieces shallow so
`cargo check` passes cleanly offline. They are marked `TODO` in code.

| Area | State in scaffold | Next step |
| --- | --- | --- |
| `whatsapp-rust` git dep | **Not pulled**. `StubWaClient` emits fake QR + echoes sends back as inbound events. | Add the git dep behind feature `wa_real`, implement `adapter.rs`. |
| SMTP sender | `SmtpSender::send_code` logs a warning + falls through to stdout sink. | Wire `AsyncSmtpTransport<Tokio1Executor>` using lettre. |
| sqlx offline metadata | All queries use dynamic `sqlx::query*` (no macros), so `.sqlx/` is not needed. | Convert to `query!`/`query_as!`; regenerate `.sqlx/` with `cargo sqlx prepare`. |
| Reply / forward WA dispatch | Persists DB row + fires event, does NOT wire the WA `reply_to_external_id`. | Lookup source `external_id`, pass via `WaCmd::SendText`. |
| Edition | Pinned to `edition = "2021"`. Plan calls for `2024`. | Bump once 1.84 stable on the target host confirms edition-2024 support. |
| Tests | Compile-time smoke unit tests per service. | Add integration tests under `tests/` using `tower::ServiceExt::oneshot`. |

## Layout

```
backend/
├── Cargo.toml                # single crate, edition 2021
├── rust-toolchain.toml       # channel = "1.84"
├── Containerfile             # multi-stage builder + debian-slim runtime
├── podman/commands.sh        # build / run / stop / logs wrapper
├── migrations/0001_init.sql  # full schema
├── .env.example              # every env var documented
└── src/
    ├── main.rs               # bootstrap + graceful shutdown
    ├── lib.rs                # build_app() facade
    ├── config.rs             # env loader
    ├── error.rs              # AppError + IntoResponse
    ├── state.rs              # AppState passed to every handler
    ├── domain/               # pure types, newtypes, zero IO
    ├── application/          # auth / wa_session / chat / realtime services
    ├── infrastructure/       # db, email, session_store, wa_client (stub)
    └── routes/               # health, auth, whatsapp, chats, ws
```

## Build + run locally (without podman)

```bash
cd backend
cp .env.example .env          # fill COOKIE_KEY etc.
cargo run
# → http://127.0.0.1:8080/api/health
```

## Build + run via Podman

```bash
cd backend
cp .env.example .env
./podman/commands.sh build
./podman/commands.sh run
./podman/commands.sh logs
# stop / nuke volumes:
./podman/commands.sh stop
./podman/commands.sh rm      # also removes data + session volumes
```

Named volumes persist across `stop/run`:
- `wa-data` → `/data` (sqlite app.db + WAL)
- `wa-session` → `/wa-session` (whatsapp-rust session blob)

## Environment variables

See `.env.example` — every var has an inline comment. Critical bits:

- `COOKIE_KEY` — 64 hex chars. Generate via `openssl rand -hex 32`.
- `FRONTEND_ORIGIN` — exact origin of the Next.js app (cookies require
  a specific origin, no wildcard).
- `EMAIL_MODE=stdout` makes magic codes show up in container logs as
  `CODE email=<x> code=<y>` — grep that during smoke tests.

## Smoke commands

```bash
# 1) Liveness
curl -fsS http://localhost:8080/api/health | jq .

# 2) Login start — then grep backend logs for the printed code
curl -fsSI -X POST http://localhost:8080/api/auth/login/start \
  -H 'content-type: application/json' \
  -d '{"email":"you@example.com"}'
./podman/commands.sh logs | grep 'CODE email='

# 3) Login verify (use the code from step 2)
curl -fsS -c /tmp/jar -X POST http://localhost:8080/api/auth/login/verify \
  -H 'content-type: application/json' \
  -d '{"email":"you@example.com","code":"123456"}' | jq .

# 4) Session echo
curl -fsS -b /tmp/jar http://localhost:8080/api/auth/session | jq .

# 5) WA stub state + QR
curl -fsS -b /tmp/jar http://localhost:8080/api/whatsapp/state | jq .
curl -fsS -b /tmp/jar http://localhost:8080/api/whatsapp/qr | jq .

# 6) Fake "connect" (stub only — flips state to Connected)
curl -fsS -b /tmp/jar -X POST http://localhost:8080/api/whatsapp/reconnect
```

## Routes (reference)

| Method | Path | Auth |
| --- | --- | --- |
| POST | `/api/auth/login/start` | none |
| POST | `/api/auth/login/verify` | none |
| POST | `/api/auth/logout` | session |
| GET  | `/api/auth/session` | session |
| GET  | `/api/whatsapp/state` | session |
| GET  | `/api/whatsapp/qr` | session |
| POST | `/api/whatsapp/reconnect` | session |
| GET  | `/api/chats` | session |
| POST | `/api/chats` | session |
| GET  | `/api/chats/:id/messages?before=&limit=` | session |
| POST | `/api/chats/:id/messages` | session |
| POST | `/api/chats/:id/messages/:mid/reply` | session |
| POST | `/api/chats/:id/messages/:mid/forward` | session |
| GET  | `/api/health` | none |
| GET  | `/ws` | session (cookie on upgrade) |

## Known blockers / things to verify when bringing this to green

- Axum 0.8 path syntax: routes use `:id` — confirm at build time; newer axum
  prefers `{id}`. If cargo errors on the path segment, swap to `{id}` form.
- `axum_extra` `PrivateCookieJar` derived from `parts.headers` — we pass the
  key explicitly, but if the axum-extra version needs a different constructor,
  adapt the `from_request_parts` impl.
- The `time` + `cookie` direct deps are pinned (`0.3`, `0.18`). If axum-extra
  pulls a conflicting major, align via a `[patch.crates-io]` entry.
- The `AuthUser` extractor uses the stable `FromRequestParts` trait (no
  `async_trait` needed on axum 0.8) — may require nightly on older toolchains.

## License

MIT — see workspace root.
