'use client';

import type { Chat, Message } from '@/lib/types';
import { fmtRelative } from '@/lib/time';
import { cn } from '@/lib/cn';

export type ChatListItemProps = {
  chat: Chat;
  lastMessage?: Message | undefined;
  active: boolean;
  onClick: () => void;
};

function initials(chat: Chat): string {
  const base = chat.display_name?.trim();
  if (base) {
    const parts = base.split(/\s+/).filter(Boolean);
    const first = parts[0]?.[0] ?? '';
    const second = parts[1]?.[0] ?? '';
    const pair = (first + second).toUpperCase();
    if (pair) return pair;
  }
  const digits = chat.wa_jid.replace(/\D/g, '');
  return digits.slice(-2) || '??';
}

export default function ChatListItem({ chat, lastMessage, active, onClick }: ChatListItemProps) {
  const preview = lastMessage?.body ?? 'Sem mensagens ainda';
  const when = fmtRelative(chat.last_message_at ?? lastMessage?.ts ?? null);
  return (
    <button
      type="button"
      onClick={onClick}
      aria-current={active ? 'page' : undefined}
      className={cn(
        'flex w-full items-center gap-3 border-b border-wa-border px-3 py-3 text-left transition hover:bg-wa-active',
        active && 'bg-wa-active',
      )}
    >
      <span
        aria-hidden="true"
        className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-wa-primary/20 text-sm font-semibold text-wa-primaryDark"
      >
        {initials(chat)}
      </span>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="flex items-baseline justify-between gap-2">
          <span className="truncate font-medium text-wa-ink">{chat.display_name || chat.wa_jid}</span>
          <span className="shrink-0 text-xs text-wa-muted">{when}</span>
        </span>
        <span className="truncate text-sm text-wa-muted">{preview}</span>
      </span>
    </button>
  );
}
