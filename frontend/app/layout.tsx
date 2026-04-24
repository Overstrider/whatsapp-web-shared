import type { Metadata } from 'next';
import './globals.css';
import type { ReactNode } from 'react';

export const metadata: Metadata = {
  title: 'WA Web Shared',
  description: 'Chat WhatsApp compartilhado',
  robots: { index: false, follow: false },
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="pt-BR">
      <body className="min-h-screen bg-wa-panelAlt text-wa-ink antialiased">{children}</body>
    </html>
  );
}
