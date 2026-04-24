#!/usr/bin/env bash
# Magic-code login smoke test. Reads code from backend stdout log.
# Arg 1: email (default: smoke@example.com)
set -euo pipefail
EMAIL="${1:-smoke@example.com}"
BASE="${BASE:-http://localhost:8080}"

echo "[1] POST /api/auth/login/start email=$EMAIL"
curl -fsS -X POST "$BASE/api/auth/login/start" \
  -H 'content-type: application/json' \
  -d "{\"email\":\"$EMAIL\"}"
echo

echo "[2] Read the 6-digit code from backend logs, then run:"
echo "    curl -c /tmp/smoke.jar -X POST $BASE/api/auth/login/verify \\"
echo "      -H 'content-type: application/json' \\"
echo "      -d '{\"email\":\"$EMAIL\",\"code\":\"<CODE>\"}'"
