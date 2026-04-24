// Env reader. Fail-fast on the client so missing config surfaces immediately.

const rawApiBase = process.env.NEXT_PUBLIC_API_BASE;
const rawWsUrl = process.env.NEXT_PUBLIC_WS_URL;

function required(name: string, value: string | undefined): string {
  if (!value || value.trim() === '') {
    // Build-time safe default — only throws in the browser at runtime.
    if (typeof window !== 'undefined') {
      throw new Error(
        `[config] Missing ${name}. Copy .env.local.example to .env.local and set it.`,
      );
    }
    return '';
  }
  return value;
}

export const config = {
  apiBase: required('NEXT_PUBLIC_API_BASE', rawApiBase) || 'http://localhost:8080',
  wsUrl: required('NEXT_PUBLIC_WS_URL', rawWsUrl) || 'ws://localhost:3000/ws',
} as const;
