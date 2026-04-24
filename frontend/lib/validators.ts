import { z } from 'zod';

// -- Core entities (mirror backend arcplan contracts) -----------------------

export const UserSchema = z.object({
  id: z.string(),
  email: z.string().email(),
  created_at: z.string(),
});

export const ChatSchema = z.object({
  id: z.string(),
  wa_jid: z.string(),
  display_name: z.string(),
  last_message_at: z.string().nullable(),
});

export const MessageDirectionSchema = z.union([z.literal('in'), z.literal('out')]);

export const MessageSchema = z.object({
  id: z.string(),
  chat_id: z.string(),
  external_id: z.string().nullable(),
  direction: MessageDirectionSchema,
  body: z.string(),
  reply_to_message_id: z.string().nullable(),
  forwarded_from_message_id: z.string().nullable(),
  ts: z.string(),
});

// -- WA connection state ---------------------------------------------------
// Backend shape: "Disconnected" | "WaitingQr" | "Connecting" | "Connected"
//              | "LoggedOut"    | { Error: "msg" }
export const WaStateSchema = z.union([
  z.literal('Disconnected'),
  z.literal('WaitingQr'),
  z.literal('Connecting'),
  z.literal('Connected'),
  z.literal('LoggedOut'),
  z.object({ Error: z.string() }),
]);

// -- WS event frames -------------------------------------------------------

export const WaStateChangeEventSchema = z.object({
  type: z.literal('WaStateChange'),
  state: WaStateSchema,
});

export const QrUpdateEventSchema = z.object({
  type: z.literal('QrUpdate'),
  qr: z.string(),
});

export const NewMessageEventSchema = z.object({
  type: z.literal('NewMessage'),
  chat_id: z.string(),
  message: MessageSchema,
});

export const ChatUpsertedEventSchema = z.object({
  type: z.literal('ChatUpserted'),
  chat: ChatSchema,
});

export const EventSchema = z.discriminatedUnion('type', [
  WaStateChangeEventSchema,
  QrUpdateEventSchema,
  NewMessageEventSchema,
  ChatUpsertedEventSchema,
]);

// -- Response envelopes ----------------------------------------------------

export const SessionResponseSchema = z.object({ user: UserSchema });
export const VerifyResponseSchema = z.object({ user: UserSchema });
export const WaStateResponseSchema = z.object({ state: WaStateSchema }).or(WaStateSchema);
export const WaQrResponseSchema = z.object({ qr: z.string() });
export const HealthResponseSchema = z.object({ ok: z.boolean(), wa: WaStateSchema });

export const ErrorBodySchema = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
  }),
});

// Validators for form input
export const EmailSchema = z.string().trim().toLowerCase().email();
export const VerifyCodeSchema = z
  .string()
  .trim()
  .regex(/^\d{6}$/, 'Código deve ter 6 dígitos');
