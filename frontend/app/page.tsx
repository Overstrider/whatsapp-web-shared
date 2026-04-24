import { redirect } from 'next/navigation';
import { cookies } from 'next/headers';
import { currentSessionServer, waStateServer } from '@/lib/api';

export const dynamic = 'force-dynamic';

export default async function RootGate() {
  const cookieStore = await cookies();
  const sid = cookieStore.get('sid')?.value;

  if (!sid) redirect('/login');

  const cookieHeader = cookieStore
    .getAll()
    .map((c) => `${c.name}=${c.value}`)
    .join('; ');

  const session = await currentSessionServer(cookieHeader);
  if (!session) redirect('/login');

  const state = await waStateServer(cookieHeader);
  if (state === 'Connected') redirect('/chat');
  redirect('/qr');
}
