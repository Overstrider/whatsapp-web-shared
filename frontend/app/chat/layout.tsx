'use client';

import type { ReactNode } from 'react';
import Sidebar from '@/components/Sidebar';
import AuthedBoot from '@/components/AuthedBoot';
import ErrorToast from '@/components/ErrorToast';
import { useRequireSession } from '@/hooks/useRequireSession';

export default function ChatLayout({ children }: { children: ReactNode }) {
  useRequireSession();
  return (
    <div className="flex h-screen w-full overflow-hidden bg-wa-panelAlt">
      <AuthedBoot />
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">{children}</div>
      <ErrorToast />
    </div>
  );
}
