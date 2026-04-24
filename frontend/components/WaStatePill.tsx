import type { WaState } from '@/lib/types';
import { cn } from '@/lib/cn';

export type WaStatePillProps = {
  state: WaState;
  className?: string;
};

type Descriptor = { dot: string; label: string; ring: string };

function describe(state: WaState): Descriptor {
  if (typeof state === 'object') {
    return { dot: 'bg-red-500', label: `Erro: ${state.Error}`, ring: 'ring-red-200' };
  }
  switch (state) {
    case 'Connected':
      return { dot: 'bg-green-500', label: 'Conectado', ring: 'ring-green-200' };
    case 'Connecting':
      return { dot: 'bg-amber-500', label: 'Conectando…', ring: 'ring-amber-200' };
    case 'WaitingQr':
      return { dot: 'bg-sky-500', label: 'Aguardando QR', ring: 'ring-sky-200' };
    case 'Disconnected':
      return { dot: 'bg-gray-400', label: 'Desconectado', ring: 'ring-gray-200' };
    case 'LoggedOut':
      return { dot: 'bg-red-500', label: 'Sessão encerrada', ring: 'ring-red-200' };
    default:
      return { dot: 'bg-gray-400', label: 'Desconhecido', ring: 'ring-gray-200' };
  }
}

export default function WaStatePill({ state, className }: WaStatePillProps) {
  const d = describe(state);
  return (
    <span
      className={cn(
        'inline-flex items-center gap-2 rounded-full bg-white/70 px-2.5 py-1 text-xs font-medium text-wa-ink ring-1',
        d.ring,
        className,
      )}
    >
      <span className={cn('h-2 w-2 rounded-full', d.dot)} aria-hidden="true" />
      {d.label}
    </span>
  );
}
