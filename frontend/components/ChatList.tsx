'use client';

import { useMemo } from 'react';
import { useRouter, useParams } from 'next/navigation';
import { useAppStore } from '@/lib/store';
import ChatListItem from './ChatListItem';

export type ChatListProps = {
  filter?: string;
};

export default function ChatList({ filter = '' }: ChatListProps) {
  const router = useRouter();
  const params = useParams<{ chatId?: string }>();
  const activeId = params?.chatId ?? null;
  const byId = useAppStore((s) => s.byId);
  const order = useAppStore((s) => s.order);
  const messagesByChat = useAppStore((s) => s.messagesByChat);
  const setActive = useAppStore((s) => s.setActive);

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    const all = order.map((id) => byId[id]).filter((c): c is NonNullable<typeof c> => !!c);
    if (!q) return all;
    return all.filter(
      (c) =>
        c.display_name.toLowerCase().includes(q) || c.wa_jid.toLowerCase().includes(q),
    );
  }, [order, byId, filter]);

  if (filtered.length === 0) {
    return (
      <div className="p-6 text-center text-sm text-wa-muted">
        Nenhuma conversa. Inicie uma nova.
      </div>
    );
  }

  return (
    <ul className="wa-scroll flex-1 overflow-y-auto">
      {filtered.map((chat) => {
        const msgs = messagesByChat[chat.id];
        const last = msgs && msgs.length > 0 ? msgs[msgs.length - 1] : undefined;
        return (
          <li key={chat.id}>
            <ChatListItem
              chat={chat}
              lastMessage={last}
              active={chat.id === activeId}
              onClick={() => {
                setActive(chat.id);
                router.push(`/chat/${chat.id}`);
              }}
            />
          </li>
        );
      })}
    </ul>
  );
}
