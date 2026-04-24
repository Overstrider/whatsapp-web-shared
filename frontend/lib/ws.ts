// WebSocket singleton with exponential backoff + dispatch into the zustand store.
// Cookie travels with the upgrade because of same-origin (dev proxy) or
// credentialed CORS (prod).

import { EventSchema } from './validators';
import { useAppStore } from './store';
import { config } from './config';
import { setUnauthorizedHandler } from './api';

const BACKOFFS = [1000, 2000, 4000, 8000, 16000, 30000] as const;
const JITTER = 0.2;

type Controller = {
  socket: WebSocket | null;
  attempt: number;
  closedByUser: boolean;
  reconnectTimer: ReturnType<typeof setTimeout> | null;
  onAuthFail: (() => void) | null;
};

const ctrl: Controller = {
  socket: null,
  attempt: 0,
  closedByUser: false,
  reconnectTimer: null,
  onAuthFail: null,
};

function jittered(ms: number): number {
  const delta = ms * JITTER;
  return Math.round(ms + (Math.random() * 2 - 1) * delta);
}

function nextDelay(): number {
  const idx = Math.min(ctrl.attempt, BACKOFFS.length - 1);
  // Safe by construction: idx clamped to [0, BACKOFFS.length-1].
  const base = BACKOFFS[idx] ?? BACKOFFS[BACKOFFS.length - 1] ?? 30_000;
  return jittered(base);
}

function scheduleReconnect(url: string): void {
  if (ctrl.reconnectTimer) return;
  if (ctrl.closedByUser) return;
  const delay = nextDelay();
  const at = Date.now() + delay;
  useAppStore.getState().setReconnectAt(at);
  ctrl.reconnectTimer = setTimeout(() => {
    ctrl.reconnectTimer = null;
    ctrl.attempt += 1;
    open(url);
  }, delay);
}

function dispatchFrame(raw: unknown): void {
  const parsed = EventSchema.safeParse(raw);
  if (!parsed.success) {
    // Drift between backend + frontend — surface loudly.
    // eslint-disable-next-line no-console
    console.warn('[ws] unknown event frame', parsed.error.issues, raw);
    useAppStore.getState().pushToast('Evento desconhecido recebido', 'error');
    return;
  }
  const ev = parsed.data;
  const store = useAppStore.getState();
  switch (ev.type) {
    case 'WaStateChange':
      store.setWaState(ev.state);
      if (ev.state !== 'WaitingQr') store.setQr(null);
      break;
    case 'QrUpdate':
      store.setQr(ev.qr);
      break;
    case 'NewMessage': {
      const existing = store.byId[ev.chat_id];
      store.upsertChat(
        existing
          ? { ...existing, last_message_at: ev.message.ts }
          : {
              id: ev.chat_id,
              wa_jid: ev.chat_id,
              display_name: ev.chat_id,
              last_message_at: ev.message.ts,
            },
      );
      store.appendMessage(ev.chat_id, ev.message);
      break;
    }
    case 'ChatUpserted':
      store.upsertChat(ev.chat);
      break;
  }
}

function open(url: string): void {
  if (typeof window === 'undefined') return;
  const store = useAppStore.getState();
  store.setStatus('connecting');
  let sock: WebSocket;
  try {
    sock = new WebSocket(url);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error('[ws] construct failed', err);
    store.setStatus('error');
    scheduleReconnect(url);
    return;
  }
  ctrl.socket = sock;

  sock.addEventListener('open', () => {
    ctrl.attempt = 0;
    useAppStore.getState().setStatus('open');
    useAppStore.getState().setReconnectAt(null);
  });

  sock.addEventListener('message', (ev) => {
    if (typeof ev.data !== 'string') return;
    try {
      const parsed: unknown = JSON.parse(ev.data);
      dispatchFrame(parsed);
    } catch {
      // ignore non-json frames
    }
  });

  sock.addEventListener('error', () => {
    useAppStore.getState().setStatus('error');
  });

  sock.addEventListener('close', (ev) => {
    ctrl.socket = null;
    useAppStore.getState().setStatus('closed');
    // 4401 = backend says "no valid session"
    if (ev.code === 4401) {
      ctrl.closedByUser = true;
      const onFail = ctrl.onAuthFail;
      useAppStore.getState().logout();
      if (onFail) onFail();
      return;
    }
    if (!ctrl.closedByUser) scheduleReconnect(url);
  });
}

export type ConnectOptions = {
  url?: string;
  onAuthFail?: () => void;
};

export function connect(opts: ConnectOptions = {}): void {
  if (typeof window === 'undefined') return;
  if (ctrl.socket && (ctrl.socket.readyState === WebSocket.OPEN || ctrl.socket.readyState === WebSocket.CONNECTING)) {
    return;
  }
  ctrl.closedByUser = false;
  ctrl.attempt = 0;
  ctrl.onAuthFail = opts.onAuthFail ?? null;
  // Also route REST 401s to the same handler.
  setUnauthorizedHandler(() => {
    ctrl.closedByUser = true;
    useAppStore.getState().logout();
    if (ctrl.onAuthFail) ctrl.onAuthFail();
  });
  open(opts.url ?? config.wsUrl);
}

export function disconnect(): void {
  ctrl.closedByUser = true;
  if (ctrl.reconnectTimer) {
    clearTimeout(ctrl.reconnectTimer);
    ctrl.reconnectTimer = null;
  }
  if (ctrl.socket) {
    try {
      ctrl.socket.close(1000, 'client-disconnect');
    } catch {
      // ignore
    }
    ctrl.socket = null;
  }
  useAppStore.getState().setStatus('idle');
  useAppStore.getState().setReconnectAt(null);
  setUnauthorizedHandler(null);
}
