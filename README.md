# BlocRoc

A decentralized event ticketing platform built on a custom [Substrate](https://substrate.io) blockchain. BlocRoc replaces paper tickets and centralized ticketing systems with on-chain NFT tickets, a trustless secondary marketplace, and a mobile validator app for venue entry.

## Repository Structure

```
blocroc/
├── roc-chain/              Substrate blockchain node
│   ├── node/               P2P node, CLI, RPC server
│   ├── runtime/            WASM runtime (FRAME-based)
│   └── pallets/
│       ├── ticket/         Core ticket pallet (mint, transfer, burn)
│       ├── event/          Event management (create, capacity, metadata)
│       ├── marketplace/    Secondary market (list, bid, buy)
│       └── scanner/        Entry validation (scan, revoke)
├── roc-frontend/           Next.js web app
│   ├── venue dashboard     Create events, manage capacity, view sales
│   └── fan marketplace     Browse events, buy/sell tickets
├── roc-scanner/            React Native mobile scanner app
│   └── entry validation    QR scan → on-chain validation → gate open/close
├── roc-indexer/            SubQuery indexer
│   └── off-chain queries   Event history, ticket ownership, sales analytics
├── docs/                   Architecture docs, API specs, ADRs
└── scripts/                Setup, dev, and demo scripts
```

## Key Features

- **On-chain tickets** — Each ticket is a unique on-chain asset owned by the purchaser's wallet
- **Trustless transfers** — Fans can resell tickets P2P without a platform intermediary
- **Controlled secondary market** — Organizers can cap resale price and take a royalty cut
- **Mobile validation** — Venue staff scan QR codes; validity is verified on-chain in real time
- **SubQuery indexing** — Rich off-chain queries for dashboards and analytics without full node access

## Technology Stack

| Layer | Technology |
|---|---|
| Blockchain | Substrate (Rust), FRAME pallets |
| Frontend | Next.js 14, TypeScript, Polkadot.js API |
| Mobile | React Native (Expo), TypeScript |
| Indexer | SubQuery, GraphQL |
| Storage | IPFS (event metadata, ticket artwork) |

## Getting Started

### Prerequisites

- Rust (stable + nightly): `rustup toolchain install nightly`
- WASM target: `rustup target add wasm32-unknown-unknown`
- Node.js 20+
- Docker (for local indexer)

### Quick Start

```bash
# Clone
git clone https://github.com/blocroc/blocroc
cd blocroc

# Run setup script (installs deps, builds chain)
./scripts/setup.sh

# Start local dev node
cd roc-chain
cargo run --bin roc-node -- --dev

# In a new terminal, start the frontend
cd roc-frontend
npm install && npm run dev

# Open http://localhost:3000
```

### Running Tests

```bash
# All pallet unit tests
cd roc-chain && cargo test

# Frontend tests
cd roc-frontend && npm test

# Scanner app tests
cd roc-scanner && npm test
```

## Documentation

- [Architecture Overview](docs/architecture.md)
- [API Specification](docs/api-spec.md)
- [Contributing Guide](docs/contributing.md)
- [Coding Conventions](CLAUDE.md)

## License

MIT
