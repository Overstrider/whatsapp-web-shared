'use client';

import { KeyboardEvent, useRef, useState } from 'react';
import { sendMessage as apiSend } from '@/lib/api';
import { useAppStore } from '@/lib/store';
import type { Message } from '@/lib/types';
import EmojiPicker from './EmojiPicker';
import { cn } from '@/lib/cn';

export type ComposerProps = {
  chatId: string;
};

function tmpId(): string {
  return `tmp-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export default function Composer({ chatId }: ComposerProps) {
  const textareaRef = useRef<HTMLTextAreaElement | null>(null);
  const appendMessage = useAppStore((s) => s.appendMessage);
  const replaceMessage = useAppStore((s) => s.replaceMessage);
  const removeMessage = useAppStore((s) => s.removeMessage);
  const pushToast = useAppStore((s) => s.pushToast);
  const [body, setBody] = useState('');
  const [pickerOpen, setPickerOpen] = useState(false);
  const [sending, setSending] = useState(false);

  function insertAtCursor(snippet: string) {
    const ta = textareaRef.current;
    if (!ta) {
      setBody((b) => b + snippet);
      return;
    }
    const start = ta.selectionStart ?? body.length;
    const end = ta.selectionEnd ?? body.length;
    const next = body.slice(0, start) + snippet + body.slice(end);
    setBody(next);
    requestAnimationFrame(() => {
      ta.focus();
      const pos = start + snippet.length;
      ta.setSelectionRange(pos, pos);
    });
  }

  async function send() {
    const trimmed = body.trim();
    if (!trimmed || sending) return;
    setSending(true);
    const id = tmpId();
    const optimistic: Message = {
      id,
      chat_id: chatId,
      external_id: null,
      direction: 'out',
      body: trimmed,
      reply_to_message_id: null,
      forwarded_from_message_id: null,
      ts: new Date().toISOString(),
    };
    appendMessage(chatId, optimistic);
    setBody('');
    try {
      const real = await apiSend(chatId, trimmed);
      replaceMessage(chatId, id, real);
    } catch (err) {
      removeMessage(chatId, id);
      const msg = err instanceof Error ? err.message : 'Falha ao enviar.';
      pushToast(msg, 'error');
      setBody(trimmed);
    } finally {
      setSending(false);
    }
  }

  function onKey(e: KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === 'Escape') {
      setPickerOpen(false);
      return;
    }
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void send();
    }
  }

  return (
    <div className="relative flex items-end gap-2 border-t border-wa-border bg-wa-panelAlt px-3 py-2">
      <button
        type="button"
        onClick={() => setPickerOpen((v) => !v)}
        aria-label="Abrir seletor de emoji"
        aria-expanded={pickerOpen}
        className={cn(
          'flex h-10 w-10 items-center justify-center rounded-full text-xl transition',
          pickerOpen ? 'bg-wa-primary text-white' : 'text-wa-muted hover:bg-white',
        )}
      >
        {'😊'}
      </button>
      <textarea
        ref={textareaRef}
        value={body}
        onChange={(e) => setBody(e.target.value)}
        onKeyDown={onKey}
        rows={1}
        placeholder="Digite uma mensagem"
        aria-label="Mensagem"
        className="max-h-40 min-h-10 flex-1 resize-none rounded-md border border-wa-border bg-white px-3 py-2 text-sm text-wa-ink shadow-sm focus:border-wa-primary focus:outline-none"
      />
      <button
        type="button"
        onClick={() => void send()}
        disabled={sending || !body.trim()}
        aria-label="Enviar mensagem"
        className="flex h-10 items-center justify-center rounded-md bg-wa-primary px-4 text-sm font-medium text-white shadow-sm transition hover:bg-wa-primaryDark disabled:cursor-not-allowed disabled:opacity-50"
      >
        Enviar
      </button>
      {pickerOpen && (
        <EmojiPicker
          onSelect={(emoji) => insertAtCursor(emoji)}
          onClose={() => setPickerOpen(false)}
        />
      )}
    </div>
  );
}
