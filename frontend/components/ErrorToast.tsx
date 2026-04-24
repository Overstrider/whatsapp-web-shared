'use client';

import { useAppStore } from '@/lib/store';
import { cn } from '@/lib/cn';

export default function ErrorToast() {
  const toasts = useAppStore((s) => s.toasts);
  const dismiss = useAppStore((s) => s.dismissToast);

  if (toasts.length === 0) return null;

  return (
    <div
      aria-live="polite"
      className="pointer-events-none fixed inset-x-0 bottom-4 z-50 flex justify-center"
    >
      <div className="flex w-full max-w-sm flex-col gap-2 px-4">
        {toasts.map((t) => (
          <div
            key={t.id}
            role="alert"
            className={cn(
              'pointer-events-auto flex items-start justify-between gap-3 rounded-md px-4 py-3 text-sm shadow-md',
              t.kind === 'error' && 'bg-red-600 text-white',
              t.kind === 'success' && 'bg-wa-primary text-white',
              t.kind === 'info' && 'bg-wa-ink text-white',
            )}
          >
            <span className="flex-1">{t.msg}</span>
            <button
              type="button"
              onClick={() => dismiss(t.id)}
              aria-label="Fechar aviso"
              className="text-white/80 hover:text-white"
            >
              {'×'}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
