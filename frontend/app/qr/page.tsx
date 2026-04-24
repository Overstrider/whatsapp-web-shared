import QRPanel from '@/components/QRPanel';
import ErrorToast from '@/components/ErrorToast';

export const metadata = { title: 'Conectar — WA Web Shared' };

export default function QrPage() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-wa-panelAlt px-4 py-8">
      <QRPanel />
      <ErrorToast />
    </main>
  );
}
