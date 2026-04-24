'use client';

import { useEffect, useMemo, useRef } from 'react';
import { getMessages } from '@/lib/api';
import { useAppStore } from '@/lib/store';
import type { Message } from '@/lib/types';
import MessageBubble from './MessageBubble';
import Composer from './Composer';
import LoadingDots from './LoadingDots';

export type ChatViewProps = {
  chatId: string;
};

export default function ChatView({ chatId }: ChatViewProps) {
  const chat = useAppStore((s) => s.byId[chatId]);
  const orderLen = useAppStore((s) => s.order.length);
  const messages = useAppStore((s) => s.messagesByChat[chatId]);
  const prependMessages = useAppStore((s) => s.prependMessages);
  const setActive = useAppStore((s) => s.setActive);
  const pushToast = useAppStore((s) => s.pushToast);

  const listRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    setActive(chatId);
  }, [chatId, setActive]);

  useEffect(() => {
    if (messages) return;
    let cancelled = false;
    (async () => {
      try {
        const fetched = await getMessages(chatId, { limit: 50 });
        if (!cancelled) prependMessages(chatId, fetched);
      } catch (err) {
        if (!cancelled) {
          const msg = err instanceof Error ? err.message : 'Falha ao carregar mensagens.';
          pushToast(msg, 'error');
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [chatId, messages, prependMessages, pushToast]);

  // Auto-scroll to bottom on new message.
  useEffect(() => {
    const el = listRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [messages?.length]);

  const quotedLookup = useMemo(() => {
    const map = new Map<string, Message>();
    if (messages) {
      for (const m of messages) map.set(m.id, m);
    }
    return map;
  }, [messages]);

  if (!chat) {
    return (
      <div className="flex h-full w-full items-center justify-center bg-wa-bg text-wa-muted">
        {orderLen === 0 ? <LoadingDots label="Carregando conversa" /> : 'Conversa não encontrada.'}
      </div>
    );
  }

  return (
    <section className="flex h-full w-full flex-col bg-wa-bg">
      <header className="flex shrink-0 items-center gap-3 border-b border-wa-border bg-wa-panel px-4 py-3">
        <span
          aria-hidden="true"
          className="flex h-10 w-10 items-center justify-center rounded-full bg-wa-primary/20 text-sm font-semibold text-wa-primaryDark"
        >
          {chat.display_name?.[0]?.toUpperCase() ?? '?'}
        </span>
        <div className="flex min-w-0 flex-col">
          <span className="truncate font-medium text-wa-ink">
            {chat.display_name || chat.wa_jid}
          </span>
          <span className="truncate text-xs text-wa-muted">{chat.wa_jid}</span>
        </div>
      </header>

      <div
        ref={listRef}
        className="wa-scroll flex-1 space-y-2 overflow-y-auto bg-wa-bg px-4 py-4"
      >
        {!messages && (
          <div className="flex justify-center pt-8">
            <LoadingDots label="Carregando mensagens" />
          </div>
        )}
        {messages?.map((m) => (
          <MessageBubble
            key={m.id}
            msg={m}
            quoted={m.reply_to_message_id ? quotedLookup.get(m.reply_to_message_id) : undefined}
          />
        ))}
        {messages && messages.length === 0 && (
          <div className="py-10 text-center text-sm text-wa-muted">
            Diga olá. Sem mensagens ainda.
          </div>
        )}
      </div>

      <Composer chatId={chatId} />
    </section>
  );
}
