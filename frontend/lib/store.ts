import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type { Chat, Message, Toast, ToastKind, User, WaState, WsStatus } from './types';

// ---- Session slice -------------------------------------------------------
export type SessionSlice = {
  user: User | null;
  waState: WaState;
  qr: string | null;
  setUser: (u: User | null) => void;
  setWaState: (s: WaState) => void;
  setQr: (q: string | null) => void;
  logout: () => void;
};

// ---- Chats slice ---------------------------------------------------------
export type ChatsSlice = {
  byId: Record<string, Chat>;
  order: string[];
  messagesByChat: Record<string, Message[]>;
  activeChatId: string | null;
  hydrateChats: (chats: Chat[]) => void;
  upsertChat: (chat: Chat) => void;
  setActive: (id: string | null) => void;
  appendMessage: (chatId: string, msg: Message) => void;
  prependMessages: (chatId: string, msgs: Message[]) => void;
  replaceMessage: (chatId: string, tmpId: string, msg: Message) => void;
  removeMessage: (chatId: string, msgId: string) => void;
};

// ---- WS slice ------------------------------------------------------------
export type WsSlice = {
  status: WsStatus;
  reconnectAt: number | null;
  setStatus: (s: WsStatus) => void;
  setReconnectAt: (t: number | null) => void;
};

// ---- UI slice (toasts) ---------------------------------------------------
export type UiSlice = {
  toasts: Toast[];
  pushToast: (msg: string, kind?: ToastKind) => void;
  dismissToast: (id: string) => void;
  clearToasts: () => void;
};

export type AppState = SessionSlice & ChatsSlice & WsSlice & UiSlice;

function sortChatIds(byId: Record<string, Chat>): string[] {
  return Object.keys(byId).sort((a, b) => {
    const ca = byId[a];
    const cb = byId[b];
    const ta = ca?.last_message_at ?? '';
    const tb = cb?.last_message_at ?? '';
    return tb.localeCompare(ta);
  });
}

let toastSeq = 0;

export const useAppStore = create<AppState>()(
  subscribeWithSelector((set, get) => ({
    // session
    user: null,
    waState: 'Disconnected',
    qr: null,
    setUser: (u) => set({ user: u }),
    setWaState: (s) => set({ waState: s }),
    setQr: (q) => set({ qr: q }),
    logout: () =>
      set({
        user: null,
        waState: 'Disconnected',
        qr: null,
        byId: {},
        order: [],
        messagesByChat: {},
        activeChatId: null,
      }),

    // chats
    byId: {},
    order: [],
    messagesByChat: {},
    activeChatId: null,
    hydrateChats: (chats) => {
      const byId: Record<string, Chat> = {};
      for (const c of chats) byId[c.id] = c;
      set({ byId, order: sortChatIds(byId) });
    },
    upsertChat: (chat) => {
      const byId = { ...get().byId, [chat.id]: chat };
      set({ byId, order: sortChatIds(byId) });
    },
    setActive: (id) => set({ activeChatId: id }),
    appendMessage: (chatId, msg) => {
      const existing = get().messagesByChat[chatId] ?? [];
      // dedup by external_id or id
      const dupe =
        existing.find((m) => m.id === msg.id) ||
        (msg.external_id
          ? existing.find((m) => m.external_id && m.external_id === msg.external_id)
          : undefined);
      if (dupe) return;
      const next = [...existing, msg];
      set({
        messagesByChat: { ...get().messagesByChat, [chatId]: next },
      });
      // bump chat order
      const chat = get().byId[chatId];
      if (chat) {
        const updated: Chat = { ...chat, last_message_at: msg.ts };
        const byId = { ...get().byId, [chatId]: updated };
        set({ byId, order: sortChatIds(byId) });
      }
    },
    prependMessages: (chatId, msgs) => {
      const existing = get().messagesByChat[chatId] ?? [];
      const seen = new Set(existing.map((m) => m.id));
      const merged = [...msgs.filter((m) => !seen.has(m.id)), ...existing];
      set({ messagesByChat: { ...get().messagesByChat, [chatId]: merged } });
    },
    replaceMessage: (chatId, tmpId, msg) => {
      const existing = get().messagesByChat[chatId] ?? [];
      const next = existing.map((m) => (m.id === tmpId ? msg : m));
      set({ messagesByChat: { ...get().messagesByChat, [chatId]: next } });
    },
    removeMessage: (chatId, msgId) => {
      const existing = get().messagesByChat[chatId] ?? [];
      set({
        messagesByChat: {
          ...get().messagesByChat,
          [chatId]: existing.filter((m) => m.id !== msgId),
        },
      });
    },

    // ws
    status: 'idle',
    reconnectAt: null,
    setStatus: (s) => set({ status: s }),
    setReconnectAt: (t) => set({ reconnectAt: t }),

    // ui
    toasts: [],
    pushToast: (msg, kind = 'info') => {
      toastSeq += 1;
      const id = `t-${Date.now()}-${toastSeq}`;
      set({ toasts: [...get().toasts, { id, msg, kind }] });
      if (typeof window !== 'undefined') {
        window.setTimeout(() => {
          set({ toasts: get().toasts.filter((t) => t.id !== id) });
        }, 4000);
      }
    },
    dismissToast: (id) => set({ toasts: get().toasts.filter((t) => t.id !== id) }),
    clearToasts: () => set({ toasts: [] }),
  })),
);
