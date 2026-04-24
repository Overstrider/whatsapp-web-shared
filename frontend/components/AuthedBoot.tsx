'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { currentSession, listChats, logout as apiLogout, waState as fetchWaState } from '@/lib/api';
import { connect, disconnect } from '@/lib/ws';
import { useAppStore } from '@/lib/store';

// Mounts the WebSocket + hydrates session + chats. Single instance inside
// /chat/layout. Unmount → tears down WS.
export default function AuthedBoot() {
  const router = useRouter();
  const setUser = useAppStore((s) => s.setUser);
  const setWaStateStore = useAppStore((s) => s.setWaState);
  const hydrateChats = useAppStore((s) => s.hydrateChats);
  const pushToast = useAppStore((s) => s.pushToast);

  useEffect(() => {
    let cancelled = false;
    async function boot() {
      try {
        const s = await currentSession();
        if (cancelled) return;
        setUser(s.user);
      } catch {
        router.replace('/login');
        return;
      }
      try {
        const st = await fetchWaState();
        if (!cancelled) setWaStateStore(st);
      } catch {
        // tolerate — WS will push state
      }
      try {
        const chats = await listChats();
        if (!cancelled) hydrateChats(chats);
      } catch {
        if (!cancelled) pushToast('Falha ao carregar conversas.', 'error');
      }
      connect({
        onAuthFail: () => {
          void apiLogout().catch(() => null);
          router.replace('/login');
        },
      });
    }
    void boot();
    return () => {
      cancelled = true;
      disconnect();
    };
  }, [router, setUser, setWaStateStore, hydrateChats, pushToast]);

  return null;
}
