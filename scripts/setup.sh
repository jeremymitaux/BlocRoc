#!/usr/bin/env bash
# BlocRoc development environment setup.
# Run once after cloning: ./scripts/setup.sh

set -euo pipefail

BOLD="\033[1m"
RESET="\033[0m"

step() { echo -e "\n${BOLD}==> $1${RESET}"; }

# ── Rust ──────────────────────────────────────────────────────────────────────

step "Checking Rust installation"
if ! command -v rustup &>/dev/null; then
    echo "rustup not found — installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
fi

step "Installing Rust toolchains"
rustup toolchain install stable
rustup toolchain install nightly
rustup default stable
rustup target add wasm32-unknown-unknown
rustup component add rust-src --toolchain nightly
rustup component add clippy rustfmt

# ── System dependencies ───────────────────────────────────────────────────────

step "Checking system dependencies"

OS="$(uname -s)"
if [[ "$OS" == "Linux" ]]; then
    if command -v apt-get &>/dev/null; then
        sudo apt-get update && sudo apt-get install -y \
            clang libssl-dev llvm libudev-dev pkg-config protobuf-compiler
    elif command -v pacman &>/dev/null; then
        sudo pacman -Syu --noconfirm clang openssl llvm udev pkgconf protobuf
    fi
elif [[ "$OS" == "Darwin" ]]; then
    if ! command -v brew &>/dev/null; then
        echo "Homebrew not found. Install it from https://brew.sh then re-run."
        exit 1
    fi
    brew install protobuf openssl
fi

# ── Build chain ───────────────────────────────────────────────────────────────

step "Building roc-chain (this takes a few minutes on first run)"
(cd roc-chain && cargo build 2>&1)

# ── Node.js ───────────────────────────────────────────────────────────────────

step "Checking Node.js"
if ! command -v node &>/dev/null; then
    echo "Node.js not found. Install Node 20+ from https://nodejs.org then re-run."
    exit 1
fi

NODE_VERSION=$(node -e "process.stdout.write(process.version.slice(1).split('.')[0])")
if [[ "$NODE_VERSION" -lt 20 ]]; then
    echo "Node.js 20+ required (found v$NODE_VERSION)."
    exit 1
fi

step "Installing roc-frontend dependencies"
(cd roc-frontend && npm install)

step "Installing roc-scanner dependencies"
(cd roc-scanner && npm install)

step "Installing roc-indexer dependencies"
(cd roc-indexer && npm install)

# ── Done ──────────────────────────────────────────────────────────────────────

echo -e "\n${BOLD}Setup complete!${RESET}"
echo ""
echo "Quick start:"
echo "  1. Start local node:   cd roc-chain && cargo run --bin roc-node -- --dev"
echo "  2. Start frontend:     cd roc-frontend && npm run dev"
echo "  3. Start indexer:      cd roc-indexer && npm run start:docker"
echo "  4. Run pallet tests:   cd roc-chain && cargo test"
