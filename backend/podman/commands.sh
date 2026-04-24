#!/usr/bin/env bash
# Wrapper around common podman operations for the backend container.
#
# Run from `backend/` directory. Expects a sibling `.env` file (copy
# `.env.example` → `.env` and fill in real values).
#
# Usage: ./podman/commands.sh {build|volumes|run|stop|restart|logs|shell|rm}

set -euo pipefail

IMAGE="whatsapp-web-shared:latest"
NAME="wa-backend"

cmd_build() {
  podman build -t "$IMAGE" -f Containerfile .
}

cmd_volumes() {
  podman volume create wa-data 2>/dev/null || true
  podman volume create wa-session 2>/dev/null || true
}

cmd_run() {
  cmd_volumes
  podman run -d --name "$NAME" \
    -p 8080:8080 \
    -v wa-data:/data \
    -v wa-session:/wa-session \
    --env-file .env \
    --restart unless-stopped \
    "$IMAGE"
}

cmd_stop() {
  podman stop "$NAME" 2>/dev/null || true
  podman rm "$NAME" 2>/dev/null || true
}

cmd_restart() {
  podman restart "$NAME"
}

cmd_logs() {
  podman logs -f "$NAME"
}

cmd_shell() {
  podman exec -it "$NAME" /bin/bash
}

cmd_rm() {
  cmd_stop
  podman volume rm wa-data wa-session 2>/dev/null || true
}

case "${1:-}" in
  build|volumes|run|stop|restart|logs|shell|rm) "cmd_$1" ;;
  *)
    echo "usage: $0 {build|volumes|run|stop|restart|logs|shell|rm}" >&2
    exit 1
    ;;
esac
