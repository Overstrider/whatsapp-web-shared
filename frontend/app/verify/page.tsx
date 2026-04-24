import { Suspense } from 'react';
import VerifyForm from '@/components/VerifyForm';
import ErrorToast from '@/components/ErrorToast';
import LoadingDots from '@/components/LoadingDots';

export const metadata = { title: 'Verificar código — WA Web Shared' };

export default function VerifyPage() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-wa-panelAlt px-4">
      <div className="w-full max-w-md rounded-lg bg-white p-8 shadow">
        <h1 className="mb-1 text-xl font-semibold text-wa-ink">Verificar código</h1>
        <p className="mb-6 text-sm text-wa-muted">Enviamos um código de 6 dígitos ao seu e-mail.</p>
        <Suspense fallback={<LoadingDots />}>
          <VerifyForm />
        </Suspense>
      </div>
      <ErrorToast />
    </main>
  );
}
