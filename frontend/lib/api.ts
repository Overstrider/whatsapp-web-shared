import ky, { type KyInstance, HTTPError } from 'ky';
import {
  ChatSchema,
  ErrorBodySchema,
  MessageSchema,
  SessionResponseSchema,
  VerifyResponseSchema,
  WaQrResponseSchema,
  WaStateResponseSchema,
  WaStateSchema,
  HealthResponseSchema,
} from './validators';
import type { Chat, Message, User, WaState } from './types';

export class AppApiError extends Error {
  readonly status: number;
  readonly code: string;
  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
    this.name = 'AppApiError';
  }
}

// Base URL: use the browser origin at runtime so Next dev rewrites can proxy
// `/api/*` to the backend (same-origin = cookie + ws friendly). On the server
// side (RSC gate) we point at the absolute backend URL.
function baseUrl(): string {
  if (typeof window !== 'undefined') return window.location.origin;
  return process.env.NEXT_PUBLIC_API_BASE ?? 'http://localhost:8080';
}

// Hook called by ws.ts on 401 to avoid a circular import.
let onUnauthorized: (() => void) | null = null;
export function setUnauthorizedHandler(fn: (() => void) | null): void {
  onUnauthorized = fn;
}

function buildClient(extraHeaders?: Record<string, string>): KyInstance {
  const prefixUrl = baseUrl();
  return ky.create({
    prefixUrl,
    credentials: 'include',
    timeout: 15_000,
    retry: {
      limit: 1,
      methods: ['get'],
      statusCodes: [408, 429, 500, 502, 503, 504],
    },
    headers: {
      'content-type': 'application/json',
      ...(extraHeaders ?? {}),
    },
    hooks: {
      afterResponse: [
        (_req, _opts, res) => {
          if (res.status === 401 && typeof window !== 'undefined' && onUnauthorized) {
            onUnauthorized();
          }
        },
      ],
    },
  });
}

const api = buildClient();

async function handle<T>(promise: Promise<Response>, parse: (v: unknown) => T): Promise<T> {
  try {
    const res = await promise;
    if (res.status === 204) return parse(undefined);
    const contentType = res.headers.get('content-type') ?? '';
    if (!contentType.includes('application/json')) return parse(undefined);
    const data: unknown = await res.json();
    return parse(data);
  } catch (err) {
    if (err instanceof HTTPError) {
      let code = 'http_error';
      let message = err.message;
      try {
        const data: unknown = await err.response.clone().json();
        const parsed = ErrorBodySchema.safeParse(data);
        if (parsed.success) {
          code = parsed.data.error.code;
          message = parsed.data.error.message;
        }
      } catch {
        // ignore — keep defaults
      }
      throw new AppApiError(err.response.status, code, message);
    }
    throw err;
  }
}

// Endpoint helpers -------------------------------------------------------

export async function loginStart(email: string): Promise<void> {
  await handle(api.post('api/auth/login/start', { json: { email } }), () => undefined);
}

export async function verifyCode(email: string, code: string): Promise<{ user: User }> {
  return handle(api.post('api/auth/login/verify', { json: { email, code } }), (v) =>
    VerifyResponseSchema.parse(v),
  );
}

export async function logout(): Promise<void> {
  await handle(api.post('api/auth/logout'), () => undefined);
}

export async function currentSession(): Promise<{ user: User }> {
  return handle(api.get('api/auth/session'), (v) => SessionResponseSchema.parse(v));
}

// Server-side variant used by the RSC gate.
export async function currentSessionServer(cookieHeader: string): Promise<{ user: User } | null> {
  const client = buildClient({ cookie: cookieHeader });
  try {
    const res = await client.get('api/auth/session');
    if (res.status === 401) return null;
    const data: unknown = await res.json();
    return SessionResponseSchema.parse(data);
  } catch (err) {
    if (err instanceof HTTPError && err.response.status === 401) return null;
    return null;
  }
}

export async function waState(): Promise<WaState> {
  return handle(api.get('api/whatsapp/state'), (v) => {
    const parsed = WaStateResponseSchema.parse(v);
    // Accept both envelope {state} and bare state.
    if (parsed && typeof parsed === 'object' && 'state' in parsed) {
      return WaStateSchema.parse((parsed as { state: WaState }).state);
    }
    return parsed as WaState;
  });
}

export async function waStateServer(cookieHeader: string): Promise<WaState | null> {
  const client = buildClient({ cookie: cookieHeader });
  try {
    const res = await client.get('api/whatsapp/state');
    if (!res.ok) return null;
    const data: unknown = await res.json();
    const parsed = WaStateResponseSchema.parse(data);
    if (parsed && typeof parsed === 'object' && 'state' in parsed) {
      return WaStateSchema.parse((parsed as { state: WaState }).state);
    }
    return parsed as WaState;
  } catch {
    return null;
  }
}

export async function waQr(): Promise<{ qr: string }> {
  return handle(api.get('api/whatsapp/qr'), (v) => WaQrResponseSchema.parse(v));
}

export async function waReconnect(): Promise<void> {
  await handle(api.post('api/whatsapp/reconnect'), () => undefined);
}

export async function listChats(): Promise<Chat[]> {
  return handle(api.get('api/chats'), (v) => ChatSchema.array().parse(v));
}

export async function getMessages(
  chatId: string,
  opts?: { before?: string; limit?: number },
): Promise<Message[]> {
  const search = new URLSearchParams();
  if (opts?.before) search.set('before', opts.before);
  if (opts?.limit) search.set('limit', String(opts.limit));
  const qs = search.toString();
  const path = `api/chats/${encodeURIComponent(chatId)}/messages${qs ? `?${qs}` : ''}`;
  return handle(api.get(path), (v) => MessageSchema.array().parse(v));
}

export async function newChat(phoneE164: string): Promise<Chat> {
  return handle(
    api.post('api/chats', { json: { phone: phoneE164 } }),
    (v) => ChatSchema.parse(v),
  );
}

export async function sendMessage(chatId: string, body: string): Promise<Message> {
  return handle(
    api.post(`api/chats/${encodeURIComponent(chatId)}/messages`, { json: { body } }),
    (v) => MessageSchema.parse(v),
  );
}

export async function replyMessage(
  chatId: string,
  mid: string,
  body: string,
): Promise<Message> {
  return handle(
    api.post(
      `api/chats/${encodeURIComponent(chatId)}/messages/${encodeURIComponent(mid)}/reply`,
      { json: { body } },
    ),
    (v) => MessageSchema.parse(v),
  );
}

export async function forwardMessage(
  chatId: string,
  mid: string,
  dstChatId: string,
): Promise<Message> {
  return handle(
    api.post(
      `api/chats/${encodeURIComponent(chatId)}/messages/${encodeURIComponent(mid)}/forward`,
      { json: { dst_chat_id: dstChatId } },
    ),
    (v) => MessageSchema.parse(v),
  );
}

export async function health(): Promise<{ ok: boolean; wa: WaState }> {
  return handle(api.get('api/health'), (v) => HealthResponseSchema.parse(v));
}
