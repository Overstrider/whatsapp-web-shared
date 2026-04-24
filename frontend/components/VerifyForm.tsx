'use client';

import {
  ClipboardEvent,
  FormEvent,
  KeyboardEvent,
  useEffect,
  useRef,
  useState,
} from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { currentSession, loginStart, verifyCode, waState as fetchWaState } from '@/lib/api';
import { useAppStore } from '@/lib/store';
import { VerifyCodeSchema } from '@/lib/validators';
import LoadingDots from './LoadingDots';

const LEN = 6;

export default function VerifyForm() {
  const router = useRouter();
  const search = useSearchParams();
  const emailFromQs = search.get('email') ?? '';
  const setUser = useAppStore((s) => s.setUser);
  const setWaStateStore = useAppStore((s) => s.setWaState);
  const pushToast = useAppStore((s) => s.pushToast);

  const [email, setEmail] = useState(emailFromQs);
  const [digits, setDigits] = useState<string[]>(Array.from({ length: LEN }, () => ''));
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [cooldown, setCooldown] = useState(0);
  const inputsRef = useRef<Array<HTMLInputElement | null>>([]);

  useEffect(() => {
    if (cooldown <= 0) return;
    const t = window.setTimeout(() => setCooldown((v) => v - 1), 1000);
    return () => window.clearTimeout(t);
  }, [cooldown]);

  useEffect(() => {
    inputsRef.current[0]?.focus();
  }, []);

  function setDigitAt(idx: number, val: string) {
    setDigits((prev) => {
      const next = [...prev];
      next[idx] = val;
      return next;
    });
  }

  function onDigitChange(idx: number, raw: string) {
    const only = raw.replace(/\D/g, '').slice(0, 1);
    setDigitAt(idx, only);
    if (only && idx < LEN - 1) inputsRef.current[idx + 1]?.focus();
  }

  function onKeyDown(idx: number, e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Backspace' && !digits[idx] && idx > 0) {
      inputsRef.current[idx - 1]?.focus();
    }
    if (e.key === 'ArrowLeft' && idx > 0) inputsRef.current[idx - 1]?.focus();
    if (e.key === 'ArrowRight' && idx < LEN - 1) inputsRef.current[idx + 1]?.focus();
  }

  function onPaste(e: ClipboardEvent<HTMLInputElement>) {
    const text = e.clipboardData.getData('text').replace(/\D/g, '').slice(0, LEN);
    if (!text) return;
    e.preventDefault();
    const next = Array.from({ length: LEN }, (_, i) => text[i] ?? '');
    setDigits(next);
    const focusIdx = Math.min(text.length, LEN - 1);
    inputsRef.current[focusIdx]?.focus();
  }

  async function onSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    const code = digits.join('');
    const parsed = VerifyCodeSchema.safeParse(code);
    if (!parsed.success) {
      setError(parsed.error.issues[0]?.message ?? 'Código inválido.');
      return;
    }
    if (!email) {
      setError('E-mail não informado.');
      return;
    }
    setSubmitting(true);
    try {
      const res = await verifyCode(email, parsed.data);
      setUser(res.user);
      // Decide next route based on WA state.
      try {
        const st = await fetchWaState();
        setWaStateStore(st);
        if (st === 'Connected') router.replace('/chat');
        else router.replace('/qr');
      } catch {
        // session is fine; fallback
        await currentSession().catch(() => null);
        router.replace('/qr');
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Falha ao verificar código.';
      setError(msg);
      pushToast(msg, 'error');
    } finally {
      setSubmitting(false);
    }
  }

  async function resend() {
    if (cooldown > 0) return;
    if (!email) {
      setError('E-mail não informado.');
      return;
    }
    try {
      await loginStart(email);
      pushToast('Código reenviado.', 'success');
      setCooldown(60);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Falha ao reenviar.';
      pushToast(msg, 'error');
    }
  }

  return (
    <form onSubmit={onSubmit} className="w-full space-y-4" noValidate>
      <div className="space-y-1.5">
        <label htmlFor="email-verify" className="block text-sm font-medium text-wa-ink">
          E-mail
        </label>
        <input
          id="email-verify"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          className="w-full rounded-md border border-wa-border bg-white px-3 py-2 text-wa-ink shadow-sm focus:border-wa-primary focus:outline-none"
          readOnly={!!emailFromQs}
        />
      </div>
      <fieldset className="space-y-1.5">
        <legend className="block text-sm font-medium text-wa-ink">Código de 6 dígitos</legend>
        <div className="flex justify-between gap-2">
          {digits.map((d, i) => (
            <input
              key={i}
              ref={(el) => {
                inputsRef.current[i] = el;
              }}
              type="text"
              inputMode="numeric"
              autoComplete="one-time-code"
              maxLength={1}
              value={d}
              onChange={(e) => onDigitChange(i, e.target.value)}
              onKeyDown={(e) => onKeyDown(i, e)}
              onPaste={onPaste}
              aria-label={`Dígito ${i + 1}`}
              className="h-12 w-10 rounded-md border border-wa-border bg-white text-center text-lg font-semibold text-wa-ink shadow-sm focus:border-wa-primary focus:outline-none"
            />
          ))}
        </div>
        {error && <p className="text-sm text-red-600">{error}</p>}
      </fieldset>
      <button
        type="submit"
        disabled={submitting}
        className="inline-flex w-full items-center justify-center rounded-md bg-wa-primary px-4 py-2 font-medium text-white shadow-sm transition hover:bg-wa-primaryDark disabled:cursor-not-allowed disabled:opacity-60"
      >
        {submitting ? <LoadingDots label="Verificando" /> : 'Verificar'}
      </button>
      <button
        type="button"
        onClick={resend}
        disabled={cooldown > 0}
        className="w-full text-center text-sm text-wa-primaryDark underline-offset-4 hover:underline disabled:cursor-not-allowed disabled:opacity-60"
      >
        {cooldown > 0 ? `Reenviar em ${cooldown}s` : 'Reenviar código'}
      </button>
    </form>
  );
}
