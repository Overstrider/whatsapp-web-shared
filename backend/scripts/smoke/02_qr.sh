#!/usr/bin/env bash
# Fetch WA state + QR payload using the cookie jar populated by 01_login.sh.
set -euo pipefail
BASE="${BASE:-http://localhost:8080}"
JAR="/tmp/smoke.jar"

[ -f "$JAR" ] || { echo "missing $JAR — run 01_login.sh + verify first"; exit 1; }

echo "[state]"
curl -fsS -b "$JAR" "$BASE/api/whatsapp/state"
echo
echo "[qr]"
curl -fsS -b "$JAR" "$BASE/api/whatsapp/qr"
echo
