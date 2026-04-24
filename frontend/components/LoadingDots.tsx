import { cn } from '@/lib/cn';

export type LoadingDotsProps = {
  className?: string;
  label?: string;
};

export default function LoadingDots({ className, label = 'Carregando' }: LoadingDotsProps) {
  return (
    <span
      role="status"
      aria-label={label}
      className={cn('inline-flex items-center gap-1', className)}
    >
      <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-wa-muted [animation-delay:-0.3s]" />
      <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-wa-muted [animation-delay:-0.15s]" />
      <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-wa-muted" />
    </span>
  );
}
