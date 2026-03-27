#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# stop-network.sh — Cleanly stop the 4-validator BlocRoc testnet
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASE_PATH="$ROOT_DIR/tmp/blocroc-testnet"
PID_FILE="$BASE_PATH/pids"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${RED}[WARN]${NC}  $*"; }

if [ ! -f "$PID_FILE" ]; then
    # Fall back to killing any roc-node processes
    info "No PID file found. Searching for roc-node processes ..."
    PIDS=$(pgrep -f "roc-node.*--validator" 2>/dev/null || true)
    if [ -z "$PIDS" ]; then
        ok "No BlocRoc nodes running."
        exit 0
    fi
    echo "$PIDS" | while read -r pid; do
        info "Sending SIGTERM to PID $pid ..."
        kill "$pid" 2>/dev/null || true
    done
    sleep 2
    ok "All roc-node processes stopped."
    exit 0
fi

info "Stopping BlocRoc testnet nodes ..."

while read -r pid name; do
    if kill -0 "$pid" 2>/dev/null; then
        info "  Stopping [$name] (PID $pid) ..."
        kill "$pid" 2>/dev/null || true
    else
        warn "  [$name] (PID $pid) already stopped."
    fi
done < "$PID_FILE"

# Give nodes time to flush and exit gracefully
sleep 2

# Force-kill any stragglers
while read -r pid name; do
    if kill -0 "$pid" 2>/dev/null; then
        warn "  [$name] (PID $pid) didn't stop gracefully — sending SIGKILL ..."
        kill -9 "$pid" 2>/dev/null || true
    fi
done < "$PID_FILE"

rm -f "$PID_FILE"

echo ""
ok "All BlocRoc testnet nodes stopped."
echo ""
echo -e "  Chain data is preserved at: $BASE_PATH"
echo -e "  To purge:  ${CYAN}rm -rf $BASE_PATH${NC}"
echo -e "  To restart: ${CYAN}./scripts/start-network.sh${NC}"
echo ""
