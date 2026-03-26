# BlocRoc Architecture Overview

## System Diagram

```
┌───────────────────────────────────────────────────────────────────┐
│                        BlocRoc Platform                           │
│                                                                   │
│  ┌─────────────────┐     ┌──────────────────┐                    │
│  │  roc-frontend   │     │   roc-scanner    │                    │
│  │  (Next.js)      │     │  (React Native)  │                    │
│  └────────┬────────┘     └────────┬─────────┘                    │
│           │  Polkadot.js API       │  Polkadot.js API             │
│           │  (WebSocket RPC)       │  (WebSocket RPC)             │
│           └───────────┬───────────┘                              │
│                       │                                           │
│           ┌───────────▼────────────┐                             │
│           │       roc-chain        │                             │
│           │  (Substrate Node)      │                             │
│           │                        │                             │
│           │  ┌──────────────────┐  │                             │
│           │  │     Runtime      │  │                             │
│           │  │  pallet-ticket   │  │                             │
│           │  │  pallet-event    │  │                             │
│           │  │  pallet-marketplace│ │                            │
│           │  │  pallet-scanner  │  │                             │
│           │  └──────────────────┘  │                             │
│           └───────────┬────────────┘                             │
│                       │  Event stream                             │
│           ┌───────────▼────────────┐                             │
│           │     roc-indexer        │                             │
│           │  (SubQuery + GraphQL)  │                             │
│           └────────────────────────┘                             │
└───────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### roc-chain

The core of the platform. A Substrate-based blockchain with a custom WASM runtime
composed of four BlocRoc pallets:

| Pallet | Responsibility |
|---|---|
| `pallet-ticket` | Mint, transfer, burn, and invalidate tickets |
| `pallet-event` | Create and manage events, track capacity |
| `pallet-marketplace` | Secondary market: list tickets for resale, buy listings |
| `pallet-scanner` | Authorize venue scanners, validate entry, record scans |

Block production uses **Aura** (round-robin, suitable for permissioned networks and
early testnets) with **GRANDPA** finality.

### roc-frontend

A Next.js 14 web application with two primary personas:

- **Venue Organizer** — creates events, sets capacity and pricing, monitors sales,
  manages scanner accounts.
- **Fan** — connects wallet, browses events, purchases primary and secondary tickets,
  lists tickets for resale.

Connects to the node via `@polkadot/api` over WebSocket. Reads off-chain data from
the SubQuery GraphQL endpoint for list views and analytics.

### roc-scanner

A React Native (Expo) mobile app for venue gate staff:

1. Staff opens app, connects their authorized scanner wallet.
2. Camera view opens for QR scanning.
3. QR decoded → `blocroc://ticket/<id>` → `pallet-scanner.validateEntry` extrinsic submitted.
4. Node validates and emits `EntryValidated` event → app shows green/red gate indicator.

### roc-indexer

A SubQuery indexer that subscribes to all BlocRoc pallet events on-chain and writes
structured records into a Postgres database. Exposes a GraphQL API used by the
frontend for:

- Event listings and search
- Ticket ownership history
- Marketplace listing aggregates
- Per-event sales analytics

---

## Data Flow: Primary Ticket Purchase

```
Fan (browser)
  │
  ├─ 1. Calls event.create_event (organizer flow, already done)
  │
  ├─ 2. Submits ticket.mint(event_id, fan_account) extrinsic
  │      signed with fan's wallet (via polkadot-js extension)
  │
  └─ 3. Node includes extrinsic in block
         ├─ TicketMinted event emitted
         ├─ Indexer picks up event → stores Ticket record in Postgres
         └─ Frontend polls GraphQL → updates fan's ticket list
```

## Data Flow: Secondary Market Sale

```
Seller (browser)
  ├─ Calls marketplace.list(ticket_id, price)
  └─ ListingListed event → indexer stores Listing

Buyer (browser)
  ├─ Queries GraphQL for active listings
  ├─ Calls marketplace.buy(listing_id)
  └─ TicketPurchased event:
       ├─ pallet-ticket updates owner
       ├─ pallet-balances transfers price
       └─ indexer updates Listing + Ticket records
```

## Data Flow: Entry Validation

```
Scanner (mobile app)
  ├─ Scans QR → decodes ticket_id
  ├─ Submits scanner.validate_entry(event_id, ticket_id)
  │    (signed with authorized scanner account)
  └─ EntryValidated event:
       ├─ pallet-ticket marks ticket as used
       ├─ indexer stores ScanRecord
       └─ app shows green indicator
```

---

## Key Design Decisions

### Why Substrate?

- Custom pallets give full control over ticket semantics (no EVM overhead).
- WASM runtime allows forkless upgrades — crucial for a live events platform.
- Native finality (GRANDPA) means `EntryValidated` is final within seconds.

### Why SubQuery over direct RPC?

- The frontend needs rich queries (filter events by date, sort listings by price)
  that are impractical over raw storage reads.
- SubQuery decouples read-path scaling from the node.

### Why Aura + GRANDPA vs. BABE + GRANDPA?

- Aura is simpler to configure and reason about for a permissioned testnet.
- BABE can be adopted later when the validator set grows.

### Off-Chain Metadata (IPFS)

Event metadata (name, venue, date, image) is too large to store on-chain.
The `metadata_cid` field in `pallet-event` stores an IPFS CID. The frontend
and indexer resolve metadata from a public IPFS gateway.
