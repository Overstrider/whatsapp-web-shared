import ChatView from '@/components/ChatView';

type Params = { chatId: string };

export default async function ChatDetailPage({ params }: { params: Promise<Params> }) {
  const { chatId } = await params;
  return <ChatView chatId={chatId} />;
}
