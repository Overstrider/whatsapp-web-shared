'use client';

import { useEffect, useRef, useState } from 'react';
import { useRouter } from 'next/navigation';
import { waQr, waReconnect, waState as fetchWaState } from '@/lib/api';
import { useAppStore } from '@/lib/store';
import WaStatePill from './WaStatePill';
import LoadingDots from './LoadingDots';

export default function QRPanel() {
  const router = useRouter();
  const waStateStore = useAppStore((s) => s.waState);
  const setWaState = useAppStore((s) => s.setWaState);
  const qr = useAppStore((s) => s.qr);
  const setQr = useAppStore((s) => s.setQr);
  const pushToast = useAppStore((s) => s.pushToast);
  const [refreshing, setRefreshing] = useState(false);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Poll QR + state every 5s while on this screen, until Connected.
  useEffect(() => {
    async function tick() {
      try {
        const st = await fetchWaState();
        setWaState(st);
        if (st === 'Connected') return;
        if (st === 'WaitingQr') {
          try {
            const res = await waQr();
            setQr(res.qr);
          } catch {
            // 409 means already connected — next tick catches it.
          }
        }
      } catch {
        // ignore transient errors
      }
    }
    void tick();
    pollRef.current = setInterval(tick, 5000);
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
      pollRef.current = null;
    };
  }, [setWaState, setQr]);

  // Jump to /chat when connection completes.
  useEffect(() => {
    if (waStateStore === 'Connected') router.replace('/chat');
  }, [waStateStore, router]);

  async function onRefresh() {
    setRefreshing(true);
    try {
      await waReconnect();
      pushToast('Solicitação de novo QR enviada.', 'info');
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Falha ao solicitar novo QR.';
      pushToast(msg, 'error');
    } finally {
      setRefreshing(false);
    }
  }

  const isDataUrl = !!qr && qr.startsWith('data:');

  return (
    <div className="mx-auto flex w-full max-w-md flex-col items-center gap-6 rounded-lg bg-white p-8 shadow">
      <div className="flex w-full items-center justify-between">
        <h1 className="text-lg font-semibold text-wa-ink">Conectar WhatsApp</h1>
        <WaStatePill state={waStateStore} />
      </div>
      <ol className="list-decimal space-y-1 self-start pl-5 text-sm text-wa-muted">
        <li>Abra o WhatsApp no celular.</li>
        <li>Vá em Aparelhos conectados → Conectar aparelho.</li>
        <li>Escaneie o código abaixo.</li>
      </ol>
      <div className="flex h-[264px] w-[264px] items-center justify-center rounded-md bg-wa-panelAlt">
        {qr ? (
          isDataUrl ? (
            // eslint-disable-next-line @next/next/no-img-element
            <img src={qr} alt="QR Code do WhatsApp" width={240} height={240} />
          ) : (
            <code className="max-w-full break-all px-3 text-center font-mono text-xs text-wa-muted">
              {qr}
            </code>
          )
        ) : (
          <LoadingDots label="Aguardando QR" />
        )}
      </div>
      <button
        type="button"
        onClick={onRefresh}
        disabled={refreshing}
        className="w-full rounded-md border border-wa-border bg-white px-4 py-2 text-sm font-medium text-wa-ink shadow-sm transition hover:bg-wa-panelAlt disabled:opacity-60"
      >
        {refreshing ? 'Solicitando…' : 'Atualizar QR'}
      </button>
    </div>
  );
}
