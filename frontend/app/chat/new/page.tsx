'use client';

import { FormEvent, useState } from 'react';
import { useRouter } from 'next/navigation';
import PhoneInput from '@/components/PhoneInput';
import LoadingDots from '@/components/LoadingDots';
import { newChat } from '@/lib/api';
import { useAppStore } from '@/lib/store';

export default function NewChatPage() {
  const router = useRouter();
  const upsertChat = useAppStore((s) => s.upsertChat);
  const setActive = useAppStore((s) => s.setActive);
  const pushToast = useAppStore((s) => s.pushToast);
  const [value, setValue] = useState('');
  const [e164, setE164] = useState<string | null>(null);
  const [valid, setValid] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  async function onSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!valid || !e164) return;
    setSubmitting(true);
    try {
      const chat = await newChat(e164);
      upsertChat(chat);
      setActive(chat.id);
      router.replace(`/chat/${chat.id}`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Falha ao iniciar conversa.';
      pushToast(msg, 'error');
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="flex h-full flex-1 items-start justify-center bg-wa-bg p-8">
      <form
        onSubmit={onSubmit}
        className="w-full max-w-md space-y-4 rounded-lg bg-white p-6 shadow"
      >
        <h2 className="text-lg font-semibold text-wa-ink">Nova conversa</h2>
        <p className="text-sm text-wa-muted">
          Informe o número em formato internacional (com DDI).
        </p>
        <PhoneInput
          value={value}
          onChange={setValue}
          onValid={(v, e164Val) => {
            setValid(v);
            setE164(e164Val);
          }}
        />
        <div className="flex items-center justify-end gap-2">
          <button
            type="button"
            onClick={() => router.back()}
            className="rounded-md border border-wa-border px-4 py-2 text-sm text-wa-ink hover:bg-wa-panelAlt"
          >
            Cancelar
          </button>
          <button
            type="submit"
            disabled={!valid || submitting}
            className="inline-flex items-center justify-center rounded-md bg-wa-primary px-4 py-2 text-sm font-medium text-white shadow-sm transition hover:bg-wa-primaryDark disabled:cursor-not-allowed disabled:opacity-60"
          >
            {submitting ? <LoadingDots label="Iniciando" /> : 'Iniciar conversa'}
          </button>
        </div>
      </form>
    </div>
  );
}
