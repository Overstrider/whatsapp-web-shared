Crie um projeto full stack com exatamente 2 pastas principais na raiz:
- backend (Rust edition 2024 + Axum + SQLite file)
- frontend (Next.js + TypeScript, layout WhatsApp Web)

Backend:
- arquitetura modular limpa, fonte única da verdade pra sessão/chats/mensagens
- única camada que fala com WhatsApp (via https://github.com/oxidezap/whatsapp-rust)
- sessão global compartilhada (não 1 por user); persiste na SQLite + volume Podman
- autenticação por email (link mágico / código), sessão server-side + cookie HttpOnly seguro, SEM senha, SEM JWT
- WebSocket/SSE para tempo real
- endpoints: login start/verify, session, whatsapp state, QR, chats list, chat messages, new chat, send/reply/forward, health

Frontend:
- Next.js + TS, interface WhatsApp-Web style
- consome APENAS API do backend (nunca WhatsApp direto)
- telas: login email, verificação código/link, QR code, layout sidebar+chat, composer com emoji
- início de conversa por número internacional (validação local)

Runtime:
- backend em container Podman (não Docker)
- Containerfile + comandos build/run/stop/restart/logs
- SQLite + sessão WhatsApp em volumes Podman persistentes
- restart NÃO destrói banco nem sessão

Escopo v1:
- APENAS texto (sem mídia, áudio, vídeo, docs, stickers, histórico antigo, backup)
- começar registro de mensagens só depois do login via QR

Funcionalidades:
- listar conversas, abrir conversa, nova conversa por telefone, enviar/reply/forward texto + emoji unicode
- tempo real pra novas msgs + estado conexão + QR
- listar/renovar/reconectar sessão WhatsApp automaticamente

Padrões: código limpo, separação domain/app/infra/routes, tipagem forte, logs úteis, env-var config, estrutura pronta pra expansão (mídia futura)

Entrega: código completo backend+frontend, README raiz, migrations, exemplo .env, comandos dev+prod
