import type { z } from 'zod';
import type {
  ChatSchema,
  EventSchema,
  MessageSchema,
  UserSchema,
  WaStateSchema,
  ErrorBodySchema,
} from './validators';

export type User = z.infer<typeof UserSchema>;
export type Chat = z.infer<typeof ChatSchema>;
export type Message = z.infer<typeof MessageSchema>;
export type WaState = z.infer<typeof WaStateSchema>;
export type Event = z.infer<typeof EventSchema>;
export type ErrorBody = z.infer<typeof ErrorBodySchema>;

export type WsStatus = 'idle' | 'connecting' | 'open' | 'closed' | 'error';
export type ToastKind = 'info' | 'error' | 'success';

export type Toast = {
  id: string;
  msg: string;
  kind: ToastKind;
};

export type MessageDirection = Message['direction'];

// Helper: short-form state tag for rendering
export function waStateTag(state: WaState): string {
  return typeof state === 'string' ? state : 'Error';
}
