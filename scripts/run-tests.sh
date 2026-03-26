#!/usr/bin/env bash
# Run all BlocRoc test suites.

set -euo pipefail

BOLD="\033[1m"
GREEN="\033[32m"
RED="\033[31m"
RESET="\033[0m"

PASS=0
FAIL=0

run() {
    local label="$1"
    shift
    echo -e "\n${BOLD}── $label${RESET}"
    if "$@"; then
        echo -e "${GREEN}✓ $label passed${RESET}"
        ((PASS++))
    else
        echo -e "${RED}✗ $label failed${RESET}"
        ((FAIL++))
    fi
}

# Rust pallet tests
run "pallet-ticket unit tests"     bash -c "cd roc-chain && cargo test -p pallet-ticket"
run "pallet-event unit tests"      bash -c "cd roc-chain && cargo test -p pallet-event"
run "pallet-marketplace unit tests" bash -c "cd roc-chain && cargo test -p pallet-marketplace"
run "pallet-scanner unit tests"    bash -c "cd roc-chain && cargo test -p pallet-scanner"

# Frontend
run "roc-frontend tests" bash -c "cd roc-frontend && npm test -- --passWithNoTests"

# Scanner app
run "roc-scanner tests" bash -c "cd roc-scanner && npm test -- --passWithNoTests"

# Indexer
run "roc-indexer tests" bash -c "cd roc-indexer && npm test -- --passWithNoTests"

echo ""
echo -e "${BOLD}Results: ${GREEN}${PASS} passed${RESET}, ${RED}${FAIL} failed${RESET}"
[[ $FAIL -eq 0 ]]
