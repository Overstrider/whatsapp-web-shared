#!/usr/bin/env bash
# Create a chat + send a message. Requires cookie jar from 01_login.sh.
set -euo pipefail
BASE="${BASE:-http://localhost:8080}"
JAR="/tmp/smoke.jar"
PHONE="${1:-+15551234567}"
BODY="${2:-hello from smoke test}"

[ -f "$JAR" ] || { echo "missing $JAR"; exit 1; }

echo "[new chat phone=$PHONE]"
CHAT=$(curl -fsS -b "$JAR" -X POST "$BASE/api/chats" \
  -H 'content-type: application/json' \
  -d "{\"phone\":\"$PHONE\"}")
echo "$CHAT"

CHAT_ID=$(echo "$CHAT" | grep -oE '"id":"[^"]+' | head -1 | cut -d'"' -f4)
[ -n "$CHAT_ID" ] || { echo "no chat id parsed"; exit 1; }

echo "[send chat_id=$CHAT_ID]"
curl -fsS -b "$JAR" -X POST "$BASE/api/chats/$CHAT_ID/messages" \
  -H 'content-type: application/json' \
  -d "{\"body\":\"$BODY\"}"
echo
