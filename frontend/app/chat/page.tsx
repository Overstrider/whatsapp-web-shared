export default function ChatEmptyPage() {
  return (
    <div className="flex h-full flex-1 flex-col items-center justify-center gap-2 bg-wa-bg p-10 text-center">
      <h2 className="text-lg font-semibold text-wa-ink">Selecione uma conversa</h2>
      <p className="max-w-sm text-sm text-wa-muted">
        Escolha uma conversa existente à esquerda ou inicie uma nova pelo botão
        <span className="mx-1 font-medium text-wa-primaryDark">+ Nova</span>.
      </p>
    </div>
  );
}
