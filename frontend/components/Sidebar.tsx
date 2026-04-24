'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { useAppStore } from '@/lib/store';
import { logout as apiLogout } from '@/lib/api';
import WaStatePill from './WaStatePill';
import ChatList from './ChatList';

export default function Sidebar() {
  const router = useRouter();
  const user = useAppStore((s) => s.user);
  const waState = useAppStore((s) => s.waState);
  const pushToast = useAppStore((s) => s.pushToast);
  const storeLogout = useAppStore((s) => s.logout);
  const [filter, setFilter] = useState('');

  async function onLogout() {
    try {
      await apiLogout();
    } catch {
      // ignore — local state still wipes
    }
    storeLogout();
    pushToast('Sessão encerrada.', 'info');
    router.replace('/login');
  }

  const initial = (user?.email?.[0] ?? '?').toUpperCase();

  return (
    <aside className="flex h-full w-full flex-col border-r border-wa-border bg-wa-panel md:w-[360px] md:shrink-0">
      <header className="flex items-center gap-3 border-b border-wa-border bg-wa-panelAlt px-4 py-3">
        <div
          aria-hidden="true"
          className="flex h-10 w-10 items-center justify-center rounded-full bg-wa-primary text-sm font-semibold text-white"
        >
          {initial}
        </div>
        <div className="flex min-w-0 flex-1 flex-col">
          <span className="truncate text-sm font-medium text-wa-ink">
            {user?.email ?? 'Sem sessão'}
          </span>
          <WaStatePill state={waState} className="mt-1 self-start" />
        </div>
        <button
          type="button"
          onClick={onLogout}
          aria-label="Sair"
          className="rounded-md px-2 py-1 text-xs text-wa-muted transition hover:bg-white hover:text-wa-ink"
        >
          Sair
        </button>
      </header>

      <div className="flex items-center gap-2 border-b border-wa-border bg-wa-panel px-3 py-2">
        <input
          type="search"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          placeholder="Buscar conversa"
          aria-label="Buscar conversa"
          className="flex-1 rounded-md bg-wa-panelAlt px-3 py-2 text-sm text-wa-ink placeholder:text-wa-muted focus:outline-none focus:ring-2 focus:ring-wa-primary/60"
        />
        <button
          type="button"
          onClick={() => router.push('/chat/new')}
          aria-label="Nova conversa"
          className="rounded-md bg-wa-primary px-3 py-2 text-sm font-medium text-white shadow-sm transition hover:bg-wa-primaryDark"
        >
          + Nova
        </button>
      </div>

      <ChatList filter={filter} />
    </aside>
  );
}
