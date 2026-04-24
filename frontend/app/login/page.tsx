import LoginForm from '@/components/LoginForm';
import ErrorToast from '@/components/ErrorToast';

export const metadata = { title: 'Entrar — WA Web Shared' };

export default function LoginPage() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-wa-panelAlt px-4">
      <div className="w-full max-w-md rounded-lg bg-white p-8 shadow">
        <h1 className="mb-1 text-xl font-semibold text-wa-ink">WA Web Shared</h1>
        <p className="mb-6 text-sm text-wa-muted">
          Entre com seu e-mail para receber um código de acesso.
        </p>
        <LoginForm />
      </div>
      <ErrorToast />
    </main>
  );
}
