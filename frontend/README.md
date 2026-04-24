# WhatsApp Web Shared — Frontend

Next.js 15 (App Router) + React 19 + TypeScript 5.7 strict + Tailwind 4 + Zustand 5.
Consumes only the backend HTTP + WebSocket API. No direct WhatsApp traffic.

## Requirements
- Node.js >= 20.11
- Backend running on `http://localhost:8080` (or override via env).

## Setup

```bash
cp .env.local.example .env.local
pnpm install   # or npm install / yarn install
pnpm dev       # http://localhost:3000
```

## Scripts
- `pnpm dev` — Next dev server (turbopack) on :3000
- `pnpm build` — production build
- `pnpm start` — serve production build on :3000
- `pnpm lint` — ESLint (next + typescript configs)
- `pnpm typecheck` — `tsc --noEmit`

## Env

```
NEXT_PUBLIC_API_BASE=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:3000/ws
```

In dev mode, `next.config.ts` rewrites `/api/*` and `/ws` to the backend so the browser
sees them as same-origin. That lets the `sid` HttpOnly cookie travel with both the
REST calls and the WebSocket upgrade without needing a permissive CORS setup.

In production you should either:
1. Put the backend on the same origin (e.g. behind a reverse proxy), or
2. Configure the backend CORS to allow the frontend origin with `credentials=true` and
   set `NEXT_PUBLIC_API_BASE` / `NEXT_PUBLIC_WS_URL` to the absolute backend URLs.

## Architecture

- Auth cookie `sid` is HttpOnly — the frontend never reads it. All `fetch`/`ky` calls
  use `credentials: 'include'`.
- The gate page `app/page.tsx` is a Server Component that checks the session and
  redirects to `/login`, `/qr`, or `/chat`.
- Realtime: a single WebSocket singleton in `lib/ws.ts` with exponential backoff +
  jitter. Close code `4401` means auth failure — the client logs out and redirects to
  `/login`.
- State: three Zustand slices (`session`, `chats`, `ws`, plus a small `ui` toast
  slice) combined into a single `useAppStore` in `lib/store.ts`.
- Validation: all API responses and WS frames are parsed through Zod schemas in
  `lib/validators.ts`. Drift = loud parse error, surfaced via the toast.

## Views
- `/` — RSC gate. Redirects based on session + WhatsApp state.
- `/login` — email form → `POST /api/auth/login/start` → `/verify`.
- `/verify` — 6-digit code form → `POST /api/auth/login/verify` → `/qr` or `/chat`.
- `/qr` — renders the pairing QR, polls state every 5s, redirects on `Connected`.
- `/chat` — sidebar + empty state.
- `/chat/[chatId]` — active conversation with composer.
- `/chat/new` — new chat from phone number (E.164).

## Pragmatic simplifications (v1)
- `whatsapp-rust` is a backend concern. Frontend only trusts the documented HTTP +
  WS contract — any backend implementation that honors it works.
- `emoji-picker-react` is loaded via `next/dynamic` with `ssr:false` to keep the
  initial bundle small.
- Phone validation client-side is cosmetic; the backend is the authority.
- Auth never touches `localStorage`. The cookie is the single source of truth.
- No tests are scaffolded in v1 (Vitest + Playwright planned for v2).

## File layout

```
frontend/
  app/
    layout.tsx            root layout, fonts, global css
    globals.css           tailwind + WA palette
    page.tsx              server component, auth gate
    login/page.tsx        LoginForm
    verify/page.tsx       VerifyForm
    qr/page.tsx           QRPanel
    chat/
      layout.tsx          Sidebar + WS mount (AuthedBoot)
      page.tsx            empty state
      [chatId]/page.tsx   ChatView
      new/page.tsx        NewChat (PhoneInput + start)
  components/
    Sidebar.tsx
    ChatList.tsx
    ChatListItem.tsx
    ChatView.tsx
    Composer.tsx
    EmojiPicker.tsx       lazy ssr:false
    QRPanel.tsx
    WaStatePill.tsx
    MessageBubble.tsx
    LoginForm.tsx
    VerifyForm.tsx
    PhoneInput.tsx
    ErrorToast.tsx
    LoadingDots.tsx
    AuthedBoot.tsx        mounts WS
  lib/
    api.ts                ky client + typed endpoints
    ws.ts                 WebSocket manager w/ backoff
    store.ts              zustand slices combined
    types.ts              TS types (zod-inferred)
    validators.ts         zod schemas
    cn.ts                 className merge
    config.ts             env reader
    phone.ts              libphonenumber-js helpers
    time.ts               relative timestamp fmt
  hooks/
    useAutoScroll.ts
    useRequireSession.ts
```

## Build

```bash
pnpm install
pnpm typecheck
pnpm build
```
