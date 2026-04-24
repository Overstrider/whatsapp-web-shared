'use client';

import { FormEvent, useState } from 'react';
import { useRouter } from 'next/navigation';
import { loginStart } from '@/lib/api';
import { EmailSchema } from '@/lib/validators';
import { useAppStore } from '@/lib/store';
import LoadingDots from './LoadingDots';

export default function LoginForm() {
  const router = useRouter();
  const pushToast = useAppStore((s) => s.pushToast);
  const [email, setEmail] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function onSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    const parsed = EmailSchema.safeParse(email);
    if (!parsed.success) {
      setError('Informe um e-mail válido.');
      return;
    }
    setSubmitting(true);
    try {
      await loginStart(parsed.data);
      router.push(`/verify?email=${encodeURIComponent(parsed.data)}`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Falha ao enviar código.';
      setError(msg);
      pushToast(msg, 'error');
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <form onSubmit={onSubmit} className="w-full space-y-4" noValidate>
      <div className="space-y-1.5">
        <label htmlFor="email" className="block text-sm font-medium text-wa-ink">
          E-mail
        </label>
        <input
          id="email"
          type="email"
          autoComplete="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          aria-describedby={error ? 'email-error' : undefined}
          aria-invalid={error ? true : undefined}
          className="w-full rounded-md border border-wa-border bg-white px-3 py-2 text-wa-ink shadow-sm focus:border-wa-primary focus:outline-none"
          placeholder="voce@exemplo.com"
          required
        />
        {error && (
          <p id="email-error" className="text-sm text-red-600">
            {error}
          </p>
        )}
      </div>
      <button
        type="submit"
        disabled={submitting}
        className="inline-flex w-full items-center justify-center rounded-md bg-wa-primary px-4 py-2 font-medium text-white shadow-sm transition hover:bg-wa-primaryDark disabled:cursor-not-allowed disabled:opacity-60"
      >
        {submitting ? <LoadingDots label="Enviando" /> : 'Enviar código'}
      </button>
      <p className="text-center text-xs text-wa-muted">
        Enviaremos um código de 6 dígitos para seu e-mail.
      </p>
    </form>
  );
}
