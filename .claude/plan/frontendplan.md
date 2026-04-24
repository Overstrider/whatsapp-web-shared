# frontendplan

## meta
repo: frontend
lang: nextjs
stack: next 15.2 app-router + react 19 + typescript 5.7 strict + tailwind 4 + zustand 5
mode: BOOTSTRAP
node: >=20.11

## Scope
Caveman: frontend ape render WA-Web UI. No WA talk direct — talk only backend API. Screens: email login, verify code/magic link, QR scan, sidebar+chat layout, composer+emoji. Real-time via WS subscribe → zustand dispatch → React rerender. Auth cookie HttpOnly server-side. v1 text only. Redirect gate: session → wa state → chats.

## Directory Layout (full tree)
```
frontend/
  package.json
  package-lock.json
  tsconfig.json
  next.config.ts
  postcss.config.mjs
  tailwind.config.ts
  eslint.config.mjs
  .gitignore
  .env.example              # NEXT_PUBLIC_API_BASE, NEXT_PUBLIC_WS_URL
  .env.local.example
  README.md
  next-env.d.ts             # generated
  public/
    favicon.ico
    logo.svg
  src/
    app/
      layout.tsx            # root shell, fonts, providers, global css
      globals.css           # tailwind directives + WA palette vars
      page.tsx              # server: check cookie → redirect /login | /qr | /chats
      not-found.tsx
      error.tsx             # client error boundary
      loading.tsx           # top-level skeleton
      login/
        page.tsx            # client: LoginForm
      verify/
        page.tsx            # client: VerifyForm (reads ?email=)
      qr/
        page.tsx            # client: QrPanel
      chats/
        layout.tsx          # sidebar + outlet, client (ws subscription root)
        page.tsx            # empty state
        [chatId]/
          page.tsx          # ChatView + Composer
        new/
          page.tsx          # NewChatDialog as full page fallback
    components/
      Sidebar.tsx
      ChatList.tsx
      ChatListItem.tsx
      ChatView.tsx
      MessageBubble.tsx
      Composer.tsx
      EmojiPicker.tsx       # lazy wrapper around emoji-picker-react
      QrPanel.tsx
      WaStatePill.tsx
      NewChatDialog.tsx
      PhoneInput.tsx
      LoginForm.tsx
      VerifyForm.tsx
      ErrorToast.tsx
      LoadingDots.tsx
      AuthedBoot.tsx        # client: mounts ws + hydrates session once
    lib/
      api.ts                # ky client, typed endpoints, 401 handler
      ws.ts                 # WebSocket singleton + backoff + dispatch
      schemas.ts            # zod: User, Chat, Message, WaState, Event
      types.ts              # exported TS types inferred from zod
      phone.ts              # libphonenumber-js parse + E.164 validate
      cn.ts                 # clsx re-export
      emoji.ts              # helper: insert at cursor in textarea
      time.ts               # relative timestamp fmt
      config.ts             # env reader, fail-fast client side
    store/
      session.ts            # zustand slice: user, waState, qr, setters
      chats.ts              # zustand slice: byId, order, messagesByChat, actions
      ws.ts                 # zustand slice: status, reconnectAt
      index.ts              # combined hook `useAppStore`
    hooks/
      useRequireSession.ts  # client redirect guard
      useAutoScroll.ts      # chat bottom-anchor
      useDebounce.ts
```

## Dependency List (exact + why)
- next 15.2.0 — App Router + RSC + server actions for auth POSTs
- react 19.0.0, react-dom 19.0.0 — required by next 15.2
- typescript 5.7.2, strict=true, noUncheckedIndexedAccess=true
- tailwindcss 4.0.0, @tailwindcss/postcss 4.0.0 — utility CSS; postcss-only pipeline (v4 way)
- zustand 5.0.2 — tiny global store, no provider
- emoji-picker-react 4.12.0 — composer picker, lazy import
- clsx 2.1.1 — className merging
- zod 3.23.8 — runtime validate API + WS frames
- ky 1.7.2 — typed fetch, credentials:'include', retry + 401 hook
- libphonenumber-js 1.11.14 — client E.164 validation (lighter than google full)
- @types/node 22.10.1, @types/react 19.0.1, @types/react-dom 19.0.1
- eslint 9.17.0, eslint-config-next 15.2.0, @typescript-eslint 8.18.0
- prettier 3.4.2, prettier-plugin-tailwindcss 0.6.9
- vitest 2.1.8, @testing-library/react 16.1.0, jsdom 25.0.1 — unit
- @playwright/test 1.49.1 — E2E smoke (optional CI)

No socket.io. Native WebSocket only. No SWR/React-Query — zustand covers server cache v1.

## Routing Flow
```
/                  RSC: read `sid` cookie → call /api/auth/session.
                   401 → redirect /login
                   ok + waState!=Connected → /qr
                   ok + Connected → /chats
/login             client. email form → POST /api/auth/login/start → push /verify?email=
/verify            client. code input. POST /api/auth/login/verify → cookie set → router.push next
/qr                client. WS QrUpdate + GET /api/whatsapp/qr fallback poll 5s. On Connected → /chats
/chats             client layout mounts AuthedBoot (opens WS). page.tsx = empty hint.
/chats/[chatId]    client. ChatView bottom-anchored + Composer
/chats/new         client. NewChatDialog fullscreen fallback (mobile)
```
Auth gate lives in `app/page.tsx` RSC + per-protected-layout re-check via server component fetching `/api/auth/session` with `cookies()` forwarded. Client layouts trust hydrated session from initial server prop.

## Rendering Strategy (per page)
- `/` Server Component, dynamic = 'force-dynamic' (cookie read).
- `/login` Client Component (form interactivity).
- `/verify` Client (reads searchParams + submits).
- `/qr` Client (WS + polling).
- `/chats/layout` Client (holds WS + store provider boundary).
- `/chats/[chatId]` Client (ws-driven).
- `app/layout` Server (fonts, meta, no state).

'use client' pushed down to smallest subtree. No client fetches inside Server Components.

## State Model (zustand)
Slice `session`:
```
{ user: User | null, waState: WaState, qr: string | null,
  setUser, setWaState, setQr, logout }
```
Slice `chats`:
```
{ byId: Record<string, Chat>, order: string[],
  messagesByChat: Record<string, Message[]>,
  activeChatId: string | null,
  upsertChat, setActive, appendMessage, prependMessages, markRead }
```
Slice `ws`:
```
{ status: 'idle'|'connecting'|'open'|'closed'|'error', reconnectAt: number | null,
  setStatus, setReconnectAt }
```
Combined via `create<AppState>()(subscribeWithSelector(...))`. No persist middleware (session via cookie).

## API Client (`src/lib/api.ts`)
ky instance:
- prefixUrl = NEXT_PUBLIC_API_BASE
- credentials: 'include'
- timeout: 15000
- hooks.beforeError → parse ErrorBody zod; throw typed AppApiError
- hooks.afterResponse → 401 → store.logout() + redirect('/login') (client only)

Typed endpoints (all return zod-parsed):
```
loginStart(email: string): Promise<void>                     // POST /api/auth/login/start
verifyCode(email: string, code: string): Promise<{user:User}>// POST /api/auth/login/verify
logout(): Promise<void>                                      // POST /api/auth/logout
currentSession(): Promise<{user:User}>                       // GET  /api/auth/session
waState(): Promise<WaState>                                  // GET  /api/whatsapp/state
waQr(): Promise<{qr:string}>                                 // GET  /api/whatsapp/qr
waReconnect(): Promise<void>                                 // POST /api/whatsapp/reconnect
listChats(): Promise<Chat[]>                                 // GET  /api/chats
getMessages(chatId, opts?:{before?:string, limit?:number}): Promise<Message[]>
newChat(phoneE164: string): Promise<Chat>                    // POST /api/chats
sendMessage(chatId, body): Promise<Message>
replyMessage(chatId, mid, body): Promise<Message>
forwardMessage(chatId, mid, dstChatId): Promise<Message>
health(): Promise<{ok:boolean, wa:WaState}>
```
Server-side (RSC) uses ky too, passing `headers: { cookie: cookies().toString() }` via explicit `serverApi()` factory.

## WebSocket (`src/lib/ws.ts`)
Singleton module with `connect(url)` / `disconnect()`.
- URL = NEXT_PUBLIC_WS_URL (e.g. ws://localhost:8080/ws).
- Cookie travels with upgrade automatically (same site or CORS-credentials).
- Reconnect backoff: delays [1000,2000,4000,8000,16000,30000] ms + ±20% jitter. Reset on open.
- On message: `JSON.parse` → `EventSchema.safeParse` → dispatch:
  - `WaStateChange` → `session.setWaState(state)` + redirect to `/chats` if on `/qr` and Connected.
  - `QrUpdate` → `session.setQr(qr)`.
  - `NewMessage` → `chats.appendMessage(chat_id, message)` + `chats.upsertChat` bump last_message_at.
  - `ChatUpserted` → `chats.upsertChat(chat)`.
- Close code 4401 → `session.logout()` + push `/login`.
- `AuthedBoot` mounts connect once. Unmount disconnects.

## UI Components (contract per component)

### Sidebar.tsx
- Props: none. Reads store.
- Renders: header (avatar initial of user.email + email truncated), WaStatePill, search input (filters ChatList client-side), "+ Nova conversa" button → router.push('/chats/new') or opens dialog, ChatList.
- Mobile: full-width; desktop ≥md: 360px fixed column.

### ChatList.tsx
- Props: `{ filter?: string }`.
- Reads: `chats.order.map(id => chats.byId[id])`, filters by display_name OR wa_jid contains filter.
- Renders: scrollable list of ChatListItem. Virtualization deferred v2 (arcplan).
- Empty state: "Nenhuma conversa. Inicie uma nova."

### ChatListItem.tsx
- Props: `{ chat: Chat, active: boolean, onClick: () => void }`.
- Renders: avatar fallback (initials from display_name OR last 2 digits of wa_jid), name, last_message preview (from messagesByChat[last]?.body, ellipsis), time (time.ts relative). Active highlights bg-[--wa-active].
- a11y: `<button>` role, aria-current=page when active.

### ChatView.tsx
- Props: `{ chatId: string }`.
- Effects: on mount, if messagesByChat[chatId] missing → `getMessages(chatId)` → `prependMessages`. Subscribe to zustand selector of messagesByChat[chatId].
- Renders: flex-col h-full: header (ChatListItem-lite of active chat) → messages scroll region (flex-1 overflow-y-auto, reverse via flex-col-reverse trick OR bottom anchor via `useAutoScroll`) → Composer.
- Messages: map → MessageBubble with `isOwn = direction==='out'`.

### MessageBubble.tsx
- Props: `{ msg: Message, onReply: (mid:string)=>void, onForward: (mid:string)=>void }`.
- Renders: bubble aligned by direction, body with preserved newlines (whitespace-pre-wrap), timestamp footer, hover menu → reply / forward buttons.
- Reply preview: if `reply_to_message_id` set, render quoted bar with that message body (look up in messagesByChat).
- Forward badge: if `forwarded_from_message_id` set, small "Encaminhada" label.

### Composer.tsx
- Props: `{ chatId: string }`.
- Local state: `body`, `replyToMid | null`, `pickerOpen`.
- Renders: reply-quote strip (if set, with close) + textarea (auto-grow max 6 lines) + emoji button + send button.
- Keys: Enter = send (if body.trim), Shift+Enter = newline, Esc = close picker + cancel reply.
- On send: optimistic append (id=tmp-uuid, direction='out', ts=now) → `sendMessage(chatId, body)` → replace tmp with server msg. On error → ErrorToast + rollback.

### EmojiPicker.tsx
- Lazy: `dynamic(() => import('emoji-picker-react'), { ssr: false, loading: () => <LoadingDots/> })`.
- Props: `{ onSelect: (emoji: string) => void, onClose: () => void }`.
- Portal'd above composer, dismiss on outside click.

### QrPanel.tsx
- Effects: if `waState === 'WaitingQr'` + `!qr` → call `waQr()` every 5s until qr arrives or state changes. WS `QrUpdate` preempts polling.
- Renders: centered card, logo, instructions, `<img src={qrBase64OrDataUrl}/>` sized 264px, countdown "expira em 20s" if known, "Atualizar QR" button → `waReconnect()`.
- On state transition to Connected → `router.replace('/chats')`.

### WaStatePill.tsx
- Props: `{ state: WaState }`.
- Renders: colored dot + label. Map:
  - Connected → green, "Conectado"
  - Connecting → amber, "Conectando…"
  - WaitingQr → blue, "Aguardando QR"
  - Disconnected → gray, "Desconectado"
  - LoggedOut → red, "Sessão encerrada"
  - Error(msg) → red, "Erro: {msg}"

### LoginForm.tsx
- Local: `email`, `submitting`, `error`.
- Submit: zod email() validate → `loginStart(email)` → router.push(`/verify?email=${encodeURIComponent(email)}`).
- a11y: label htmlFor, aria-describedby=error.

### VerifyForm.tsx
- Local: 6-char `code` via 6 segmented inputs (auto-advance), `email` from searchParams.
- Submit: `verifyCode(email, code)` → on success `currentSession()` + `waState()` → route to `/qr` or `/chats`.
- "Reenviar código" link → `loginStart(email)` with 60s cooldown.

### PhoneInput.tsx
- Props: `{ value, onChange, onValid(valid:boolean) }`.
- Renders: country-code select (default +55) + number input. On change: `parsePhoneNumberFromString(full, country)` → E.164. Surface inline invalid.

### NewChatDialog.tsx
- Modal (radix-free; simple portal + focus trap via `inert` on siblings).
- Uses PhoneInput. Submit → `newChat(e164)` → `chats.upsertChat` + `setActive(chat.id)` + router.push(`/chats/${chat.id}`).
- Dup chat: if backend returns existing → same redirect (idempotent).

### ErrorToast.tsx
- Zustand `ui.toast` slice (mini, inside store/index) OR local context. Pick: store slice `ui: { toast:{msg,kind}|null, setToast, clear }`.
- Auto-clear after 4s.

### AuthedBoot.tsx
- Mount in `app/chats/layout.tsx`.
- Effects: `currentSession()` → hydrate user → `waState()` → hydrate → `listChats()` → populate → `ws.connect(NEXT_PUBLIC_WS_URL)`.
- Unmount: `ws.disconnect()`.

## Tailwind Theme (`tailwind.config.ts`)
- content: `./src/**/*.{ts,tsx}`.
- darkMode: `class`.
- theme.extend.colors:
  - wa.primary: #00a884
  - wa.primaryDark: #008069
  - wa.bg: #efeae2
  - wa.panel: #ffffff
  - wa.panelAlt: #f0f2f5
  - wa.ink: #111b21
  - wa.muted: #667781
  - wa.bubbleOut: #d9fdd3
  - wa.bubbleIn: #ffffff
  - wa.active: #f0f2f5
  - wa.border: #e9edef
- fontFamily.sans: ['"Segoe UI"','Roboto','system-ui','sans-serif'].
- custom utility `bubble-tail-left/right` via `@layer components` in globals.css.

## Accessibility
- Focus rings: `focus-visible:ring-2 ring-wa-primary`.
- All icon-only buttons: `aria-label` pt-BR.
- Keyboard nav: ChatList items are `<button>`, arrow-up/down moves focus (useEffect listener at list root).
- Esc closes emoji picker + NewChatDialog.
- Color contrast: labels use wa.ink on wa.panel ≥ 7:1.
- prefers-reduced-motion: disable bubble slide-in.

## Forms (>5 fields breakdown)
- Login = 1 field. N/A.
- Verify = 6-digit code (one logical field). N/A.
- NewChat = 2 fields. N/A.
None exceed 5 — no multi-step required.

## SEO / Metadata
- `app/layout.tsx`: `metadata = { title:'WA Web Shared', description:'Chat WhatsApp compartilhado', robots:{index:false,follow:false} }`.
- No public pages — robots noindex everywhere.

## Performance
- emoji-picker-react lazy via `next/dynamic` ssr:false.
- Images: QR via plain `<img>` (dynamic base64, not worth next/image).
- Fonts: `next/font/google` Inter swap=swap, subset=latin.
- Avoid re-render storms: zustand selectors per slice, shallow equality.
- WS messages throttled? No — text only, low volume.
- Bundle budget goal: initial JS < 180KB gz.

## Testing Approach
- Vitest unit:
  - `lib/schemas` happy + fail paths.
  - `lib/phone` E.164 parse variants.
  - `store/chats` actions (upsert, appendMessage, dedupe by external_id).
  - `lib/ws` dispatch mapping (mock socket).
- Testing-library component:
  - LoginForm submit → calls api.loginStart + redirects.
  - VerifyForm 6-digit paste fills all.
  - Composer Enter sends, Shift+Enter newline.
- Playwright E2E smoke (opt):
  - /login → type email → /verify → mock backend 200 → /qr page renders.

## Env vars
`.env.local.example`:
```
NEXT_PUBLIC_API_BASE=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws
```
`config.ts` asserts non-empty at module load (client). Throw with readable msg.

## Implementation Order (topo for task-architect)
1. Scaffold: package.json, tsconfig (strict), next.config.ts, tailwind.config.ts, postcss.config.mjs, eslint, prettier, globals.css, .env.example.
2. `lib/config` + `lib/schemas` + `lib/types` + `lib/cn` + `lib/time` + `lib/phone`.
3. `lib/api` (ky client + endpoints + 401 hook).
4. `store/session` + `store/chats` + `store/ws` + `store/index`.
5. `app/layout.tsx` (fonts, metadata, body classes) + `globals.css`.
6. `app/page.tsx` RSC gate + redirect logic.
7. `LoadingDots` + `ErrorToast` + `WaStatePill`.
8. `LoginForm` + `app/login/page.tsx`.
9. `VerifyForm` + `app/verify/page.tsx`.
10. `lib/ws` (singleton + backoff + dispatch).
11. `AuthedBoot`.
12. `QrPanel` + `app/qr/page.tsx`.
13. `Sidebar` + `ChatList` + `ChatListItem` + `app/chats/layout.tsx` + `app/chats/page.tsx`.
14. `PhoneInput` + `NewChatDialog` + `app/chats/new/page.tsx`.
15. `MessageBubble` + `ChatView` + `app/chats/[chatId]/page.tsx`.
16. `Composer` + `EmojiPicker` (lazy).
17. Hook: `useAutoScroll`, `useRequireSession`, `useDebounce`.
18. Unit + component tests.
19. E2E smoke (opt).
20. README + .env.local.example + run scripts.

## Cross-repo contract lock (mirror arcplan)
- User, Chat, Message, WaState, Event JSON shapes → zod schemas in `lib/schemas.ts`. Single source mirrors backend. On drift → zod parse fails → ErrorToast + telemetry.
- Cookie `sid` HttpOnly — frontend never reads it; relies on `credentials:'include'`.
- CORS origin must match NEXT_PUBLIC_API_BASE host.

## Risks / Traps (caveman)
- next 15 RSC + cookies → never touch `cookies()` outside server components / route handlers. Auth gate stays in `app/page.tsx` + re-check via server helper.
- Client Component drift → keep `'use client'` at page level only where interactive; layouts server where possible (but `/chats/layout` needs WS → client).
- WS reconnect thrash → backoff + jitter + cap. Stop retrying on 4401.
- tailwind 4 vs 3 → v4 uses `@import "tailwindcss"` in globals.css + `@tailwindcss/postcss` plugin. No `tailwind.config` auto-discovery same way — pin exact.
- emoji-picker-react heavy (~350KB) → lazy ssr:false mandatory. Gate behind user tap.
- libphonenumber-js metadata → use `/min` build to cut size.
- E.164 edge: long country codes (e.g. +1-xxx), leading zero trim → trust parsePhoneNumberFromString output only.
- Optimistic send race: WS NewMessage may arrive before POST response. Dedup by `external_id` AND tmp-id swap (store.appendMessage checks external_id set).
- QR image format: if backend returns raw string (pairing code) vs data URL → detect `startsWith('data:')` else treat as text for QR lib (v1 backend returns string per arcplan — render via `qrcode` lib? No. Backend produces PNG data URL per our contract clarification → src={qr} direct). Assume data URL.
- Cookie on WS upgrade: browser sends cookie only if same-origin OR CORS with credentials + server origin whitelist. Dev: proxy via `next.config.ts rewrites` `/api/* → API_BASE` + `/ws → WS_URL` to stay same-origin. Pin this in scaffold.
- App Router server-action POST vs client ky: pick client ky for login/verify to keep cookie flow uniform + simpler error handling.
- `noUncheckedIndexedAccess` will force optional chains on `messagesByChat[id]` — plan store selectors accordingly.

## Dev scripts (`package.json`)
```
"scripts": {
  "dev": "next dev --turbopack -p 3000",
  "build": "next build",
  "start": "next start -p 3000",
  "lint": "next lint",
  "typecheck": "tsc --noEmit",
  "test": "vitest run",
  "test:watch": "vitest",
  "e2e": "playwright test"
}
```

## next.config.ts (key bits)
- `experimental.reactCompiler = false` (stability v1).
- `rewrites` dev-only: `/api/:path* → ${API_BASE}/api/:path*`, `/ws → ${WS_URL}` for same-origin cookies.
- `images.remotePatterns = []` (no remote images v1).

PLAN_COMPLETE: frontendplan.md
