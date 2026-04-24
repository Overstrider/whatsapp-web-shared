'use client';

import type { Message } from '@/lib/types';
import { fmtClock } from '@/lib/time';
import { cn } from '@/lib/cn';

export type MessageBubbleProps = {
  msg: Message;
  quoted?: Message | undefined;
  onReply?: ((mid: string) => void) | undefined;
  onForward?: ((mid: string) => void) | undefined;
};

export default function MessageBubble({ msg, quoted, onReply, onForward }: MessageBubbleProps) {
  const isOwn = msg.direction === 'out';
  return (
    <div className={cn('group flex w-full', isOwn ? 'justify-end' : 'justify-start')}>
      <div
        className={cn(
          'relative max-w-[75%] rounded-lg px-3 py-2 shadow-sm',
          isOwn
            ? 'rounded-tr-sm bg-wa-bubbleOut text-wa-ink'
            : 'rounded-tl-sm bg-wa-bubbleIn text-wa-ink',
        )}
      >
        {msg.forwarded_from_message_id && (
          <div className="mb-1 text-xs italic text-wa-muted">Encaminhada</div>
        )}
        {quoted && (
          <div className="mb-1 rounded border-l-4 border-wa-primary/60 bg-black/5 px-2 py-1 text-xs text-wa-muted">
            <div className="truncate whitespace-pre-wrap">{quoted.body}</div>
          </div>
        )}
        <div className="whitespace-pre-wrap break-words text-sm">{msg.body}</div>
        <div className="mt-1 flex items-center justify-end gap-2 text-[10px] text-wa-muted">
          <span>{fmtClock(msg.ts)}</span>
        </div>
        {(onReply || onForward) && (
          <div
            className={cn(
              'absolute -top-3 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100',
              isOwn ? 'right-2' : 'left-2',
            )}
          >
            {onReply && (
              <button
                type="button"
                onClick={() => onReply(msg.id)}
                aria-label="Responder"
                className="rounded bg-white px-2 py-0.5 text-[10px] text-wa-ink shadow"
              >
                Responder
              </button>
            )}
            {onForward && (
              <button
                type="button"
                onClick={() => onForward(msg.id)}
                aria-label="Encaminhar"
                className="rounded bg-white px-2 py-0.5 text-[10px] text-wa-ink shadow"
              >
                Encaminhar
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
