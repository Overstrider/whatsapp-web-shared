'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAppStore } from '@/lib/store';

// Client-side guard: if user becomes null (e.g. WS 4401), redirect to /login.
export function useRequireSession(): void {
  const router = useRouter();
  const user = useAppStore((s) => s.user);
  useEffect(() => {
    if (user === null) {
      router.replace('/login');
    }
  }, [user, router]);
}
