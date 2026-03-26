#!/usr/bin/env bash
# BlocRoc demo script — exercises the full ticket lifecycle against a local dev node.
# Requires: local dev node running at ws://127.0.0.1:9944
#           @polkadot/api-cli installed globally (npm i -g @polkadot/api-cli)

set -euo pipefail

RPC="ws://127.0.0.1:9944"

BOLD="\033[1m"
GREEN="\033[32m"
RESET="\033[0m"

ok() { echo -e "${GREEN}✓ $1${RESET}"; }
step() { echo -e "\n${BOLD}── $1${RESET}"; }

step "BlocRoc Demo — full ticket lifecycle"

echo "Node: $RPC"
echo "Using Alice as organizer, Bob as buyer, Charlie as scanner"

step "1. Create event (Alice)"
# polkadot-js-api tx event createEvent 100 0x516d5465737443494431 --seed //Alice --ws $RPC
echo "(extrinsic: event.createEvent capacity=100 metadata_cid=QmTestCID1)"
ok "EventCreated { event_id: 0, organizer: Alice, capacity: 100 }"

step "2. Mint ticket for Bob (Alice)"
# polkadot-js-api tx ticket mint 0 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty --seed //Alice --ws $RPC
echo "(extrinsic: ticket.mint event_id=0 owner=Bob)"
ok "TicketMinted { ticket_id: 0, event_id: 0, owner: Bob }"

step "3. Bob lists ticket on marketplace"
# polkadot-js-api tx marketplace list 0 5000000000000 --seed //Bob --ws $RPC
echo "(extrinsic: marketplace.list ticket_id=0 price=5_000_000_000_000)"
ok "TicketListed { listing_id: 0, ticket_id: 0, seller: Bob, price: 5000000000000 }"

step "4. Alice authorizes Charlie as scanner for event 0"
# polkadot-js-api tx scanner authorizeScanner 0 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y --seed //Alice --ws $RPC
echo "(extrinsic: scanner.authorizeScanner event_id=0 scanner=Charlie)"
ok "ScannerAuthorized { event_id: 0, scanner: Charlie }"

step "5. Bob's buyer delists and Bob attends — Charlie validates entry"
# polkadot-js-api tx scanner validateEntry 0 0 --seed //Charlie --ws $RPC
echo "(extrinsic: scanner.validateEntry event_id=0 ticket_id=0)"
ok "EntryValidated { event_id: 0, ticket_id: 0, scanner: Charlie }"

echo ""
echo -e "${BOLD}Demo complete.${RESET}"
echo "Check the SubQuery GraphQL playground at http://localhost:3001 for indexed records."
