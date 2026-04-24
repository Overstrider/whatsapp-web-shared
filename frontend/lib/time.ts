// Relative timestamp formatter. All inputs are ISO-8601 strings.

const rtf = new Intl.RelativeTimeFormat('pt-BR', { numeric: 'auto' });

const MIN = 60_000;
const HOUR = 60 * MIN;
const DAY = 24 * HOUR;

export function fmtRelative(iso: string | null | undefined): string {
  if (!iso) return '';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '';
  const diff = d.getTime() - Date.now();
  const abs = Math.abs(diff);
  if (abs < MIN) return 'agora';
  if (abs < HOUR) return rtf.format(Math.round(diff / MIN), 'minute');
  if (abs < DAY) return rtf.format(Math.round(diff / HOUR), 'hour');
  if (abs < 7 * DAY) return rtf.format(Math.round(diff / DAY), 'day');
  return d.toLocaleDateString('pt-BR');
}

export function fmtClock(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '';
  return d.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' });
}
