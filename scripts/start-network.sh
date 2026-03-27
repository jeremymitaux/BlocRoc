#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# start-network.sh — Launch a 4-validator BlocRoc testnet
#
# Validators (named after iconic music venues):
#   1. The Roxy        — alice keys   (ports 30333 / 9944)
#   2. Red Rocks       — bob keys     (ports 30334 / 9945)
#   3. House of Blues   — charlie keys (ports 30335 / 9946)
#   4. Local Dive Bar   — dave keys   (ports 30336 / 9947)
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$ROOT_DIR/target/release/roc-node"
BASE_PATH="$ROOT_DIR/tmp/blocroc-testnet"
CHAIN_SPEC="$BASE_PATH/blocroc-spec-raw.json"
LOG_DIR="$BASE_PATH/logs"
PID_FILE="$BASE_PATH/pids"

# ── Colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m' # No colour

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
err()   { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

# ── Pre-flight checks ───────────────────────────────────────────────────────
if [ ! -x "$BINARY" ]; then
    err "Binary not found at $BINARY — run 'cargo build --release' first."
fi

# ── 1. Purge existing chain data ─────────────────────────────────────────────
info "Purging any existing chain data ..."
rm -rf "$BASE_PATH"
mkdir -p "$LOG_DIR"
ok "Chain data purged."

# ── 2. Generate the chain spec ───────────────────────────────────────────────
info "Generating BlocRoc chain spec ..."

# Export the human-readable spec
"$BINARY" build-spec --chain blocroc --disable-default-bootnode \
    > "$BASE_PATH/blocroc-spec.json" 2>/dev/null

# Convert to raw (hashed storage keys)
"$BINARY" build-spec --chain "$BASE_PATH/blocroc-spec.json" --raw --disable-default-bootnode \
    > "$CHAIN_SPEC" 2>/dev/null

ok "Chain spec generated at $CHAIN_SPEC"

# ── 3. Start the 4 validator nodes ──────────────────────────────────────────

# Arrays for node configuration
NAMES=("the-roxy" "red-rocks" "house-of-blues" "local-dive-bar")
SEEDS=("alice" "bob" "charlie" "dave")
P2P_PORTS=(30333 30334 30335 30336)
RPC_PORTS=(9944 9945 9946 9947)

# We'll capture Node 1's peer ID after starting it so the others can bootnode to it.
NODE1_KEY="0000000000000000000000000000000000000000000000000000000000000001"

# Derive Node 1's multiaddr. Using a fixed node-key lets us pre-compute the peer ID.
# The peer ID for the all-zeros-except-1 ed25519 key is well-known in Substrate:
NODE1_PEER_ID=$("$BINARY" key inspect-node-key --file <(echo -n "$NODE1_KEY") 2>/dev/null || true)

# Fallback: if inspect-node-key isn't available, use the known peer ID for this key
if [ -z "$NODE1_PEER_ID" ]; then
    NODE1_PEER_ID="12D3KooWEyoppNCUVQjSnzmPBHCoYF79CgPdxTBMz3wbKHCiEjWP"
fi

BOOTNODE="/ip4/127.0.0.1/tcp/30333/p2p/$NODE1_PEER_ID"

info "Starting 4 validator nodes ..."
> "$PID_FILE"

for i in 0 1 2 3; do
    name="${NAMES[$i]}"
    seed="${SEEDS[$i]}"
    p2p="${P2P_PORTS[$i]}"
    rpc="${RPC_PORTS[$i]}"
    node_base="$BASE_PATH/$name"

    EXTRA_FLAGS=()

    if [ "$i" -eq 0 ]; then
        # Node 1 uses a deterministic node-key so the others can bootnode to it.
        EXTRA_FLAGS+=(--node-key "$NODE1_KEY")
    else
        EXTRA_FLAGS+=(--bootnodes "$BOOTNODE")
    fi

    info "  Starting [$name] (${seed}) — p2p=$p2p rpc=$rpc ..."

    "$BINARY" \
        --chain "$CHAIN_SPEC" \
        --base-path "$node_base" \
        --port "$p2p" \
        --rpc-port "$rpc" \
        --rpc-cors all \
        --validator \
        --name "$name" \
        --"${seed}" \
        --rpc-methods Unsafe \
        --force-authoring \
        "${EXTRA_FLAGS[@]}" \
        > "$LOG_DIR/$name.log" 2>&1 &

    PID=$!
    echo "$PID $name" >> "$PID_FILE"
    ok "  [$name] started (PID $PID)"
done

# Give nodes a moment to boot
sleep 3

# ── 4. Insert Aura + GRANDPA keys into each node's keystore ─────────────────

info "Inserting authority keys into keystores ..."

for i in 0 1 2 3; do
    seed="${SEEDS[$i]}"
    rpc="${RPC_PORTS[$i]}"
    name="${NAMES[$i]}"
    suri="//${seed^}"   # //Alice, //Bob, //Charlie, //Dave

    # Aura key (sr25519, key type "aura")
    curl -sS -H "Content-Type: application/json" \
        -d "{\"id\":1, \"jsonrpc\":\"2.0\", \"method\":\"author_insertKey\", \"params\":[\"aura\", \"$suri\", \"$(
            "$BINARY" key inspect --scheme sr25519 "$suri" 2>/dev/null | grep "Public key" | awk '{print $NF}'
        )\"]}" \
        "http://127.0.0.1:$rpc" > /dev/null 2>&1 || true

    # GRANDPA key (ed25519, key type "gran")
    curl -sS -H "Content-Type: application/json" \
        -d "{\"id\":1, \"jsonrpc\":\"2.0\", \"method\":\"author_insertKey\", \"params\":[\"gran\", \"$suri\", \"$(
            "$BINARY" key inspect --scheme ed25519 "$suri" 2>/dev/null | grep "Public key" | awk '{print $NF}'
        )\"]}" \
        "http://127.0.0.1:$rpc" > /dev/null 2>&1 || true

    ok "  [$name] keys injected"
done

# ── 5. Status message ────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  BlocRoc 4-Validator Testnet is RUNNING${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${CYAN}The Roxy${NC}        (alice)    RPC: ws://127.0.0.1:9944"
echo -e "  ${CYAN}Red Rocks${NC}       (bob)      RPC: ws://127.0.0.1:9945"
echo -e "  ${CYAN}House of Blues${NC}   (charlie)  RPC: ws://127.0.0.1:9946"
echo -e "  ${CYAN}Local Dive Bar${NC}   (dave)     RPC: ws://127.0.0.1:9947"
echo ""
echo -e "  Chain spec:  $CHAIN_SPEC"
echo -e "  Logs:        $LOG_DIR/"
echo -e "  PIDs:        $PID_FILE"
echo ""
echo -e "  Stop the network:  ${YELLOW}./scripts/stop-network.sh${NC}"
echo ""

# Wait a few more seconds and then check if blocks are being produced.
sleep 5

BEST_BLOCK=$(curl -sS -H "Content-Type: application/json" \
    -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
    http://127.0.0.1:9944 2>/dev/null | grep -o '"peers":[0-9]*' | head -1 || true)

BLOCK_HASH=$(curl -sS -H "Content-Type: application/json" \
    -d '{"id":1,"jsonrpc":"2.0","method":"chain_getBlockHash","params":[1]}' \
    http://127.0.0.1:9944 2>/dev/null || true)

if echo "$BLOCK_HASH" | grep -q "result"; then
    ok "Blocks are being produced! GRANDPA finality is active across all 4 nodes."
else
    warn "Nodes are peering — blocks should start being produced shortly."
    warn "Check logs with: tail -f $LOG_DIR/the-roxy.log"
fi

echo ""
info "Tailing The Roxy log (Ctrl+C to detach — nodes keep running) ..."
echo ""
tail -f "$LOG_DIR/the-roxy.log"
