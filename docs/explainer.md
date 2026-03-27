# What is BlocRoc? A Plain-English Guide

## The Problem We're Solving

You've bought a concert ticket online. A few things might have gone wrong:

- The site charged you a $30 "service fee" on top of a $50 ticket
- You tried to resell it and the platform took another cut — or banned resale entirely
- You showed up at the gate, the scanner glitched, and the staff had no way to verify your ticket was real
- Someone sold you a fake

All of these problems exist because **a central company controls the ticketing system**. They set the fees, they decide the rules, and you just have to trust them.

BlocRoc fixes this by putting tickets on a blockchain.

---

## What's a Blockchain, in Plain English?

Imagine a notebook that thousands of people have a copy of. Every time something is written in it — "Alice bought ticket #42" — everyone's copy updates at the same time. Nobody can erase or change an old entry, and no single person owns the notebook.

That's a blockchain. BlocRoc has its own custom blockchain called **roc-chain**.

When you buy a ticket on BlocRoc:
- A record is written into this shared notebook: "This ticket belongs to you"
- Nobody can fake that record or delete it
- Anyone can verify it is real just by looking at the notebook

---

## How BlocRoc Works, Step by Step

### 1. An organizer creates an event

A venue or promoter connects their wallet and creates an event on BlocRoc. They set:
- The event name and date (stored on IPFS, a decentralized file system)
- How many tickets exist (capacity)
- The price

This creates a record on the blockchain that says: *"Event #7 exists. It has 500 tickets. None have been sold yet."*

### 2. A fan buys a ticket

The fan connects their crypto wallet and purchases a ticket. The blockchain record updates instantly:
- *"Ticket #312 belongs to wallet address 0xABC..."*

The ticket is now a digital asset in the fan's wallet — like a coin, but shaped like a concert ticket. It cannot be faked, duplicated, or taken away.

### 3. The fan wants to resell

Life happens. The fan can list their ticket on the BlocRoc marketplace. Another fan buys it. The blockchain records the new owner. The original organizer can set a rule like "resale price cannot exceed 120% of face value" — and that rule is enforced automatically by the code, not by a company.

### 4. The fan arrives at the venue

The venue staff use the **BlocRoc Scanner app** on their phone. They scan the QR code on the fan's ticket. The app checks the blockchain in real time:

- Does this ticket exist? ✓
- Has it already been scanned tonight? ✗ (if yes → reject)
- Is this wallet the current owner? ✓

If everything checks out, the gate opens. The scan is recorded on the blockchain so the same ticket cannot be used twice.

---

## The Four Building Blocks (Pallets)

BlocRoc's blockchain logic is split into four modules called **pallets**. Think of them like departments:

| Pallet | What it does | Real-world equivalent |
|---|---|---|
| **Ticket** | Creates, transfers, and destroys tickets | The ticket itself |
| **Event** | Manages event records and capacity | Box office database |
| **Marketplace** | Lists and sells tickets on the secondary market | StubHub / Viagogo |
| **Scanner** | Validates entry at the gate | The handheld scanner at the door |

Each pallet only knows its own job. They talk to each other through defined interfaces, like departments in a company communicating through official channels.

---

## The Four Apps in This Project

```
BlocRoc
│
├── roc-chain        The blockchain itself (written in Rust)
│                   This is the "notebook" — it runs the rules and stores the truth
│
├── roc-frontend     A website (built with Next.js)
│                   Organizers manage events here; fans browse and buy tickets
│
├── roc-scanner      A phone app (built with React Native)
│                   Venue staff use this to scan tickets at the door
│
└── roc-indexer      A search layer (built with SubQuery)
                    The blockchain stores facts but isn't great at search queries
                    like "show me all events this weekend". This layer reads the
                    blockchain and builds a searchable database for the website.
```

---

## Why Build a Custom Blockchain?

BlocRoc uses **Substrate**, a framework for building blockchains, rather than deploying on an existing chain like Ethereum.

Why? Three reasons:

1. **Full control over the rules.** We can enforce things like resale price caps directly in the blockchain's code — not as a smart contract that can be worked around, but as a fundamental law of the chain.

2. **No gas fee surprises.** On Ethereum, every transaction costs variable "gas" fees that spike unpredictably. On roc-chain we set the fee model ourselves.

3. **Forkless upgrades.** The blockchain's logic compiles to a small program (WebAssembly) stored *inside* the chain. To update the rules, we just replace that program on-chain with a governance vote — no hard fork, no node restarts.

---

## Who Does What

| Role | Tool they use | What they can do |
|---|---|---|
| **Event organizer** | roc-frontend (website) | Create events, set capacity & prices, authorize gate staff |
| **Fan** | roc-frontend (website) | Buy primary tickets, list/buy on secondary market |
| **Gate staff** | roc-scanner (phone app) | Scan tickets at entry — app gives green/red in real time |
| **Developer** | roc-chain (Rust code) | Modify the rules of the blockchain itself |

---

## The Technology, One Line Each

- **Substrate / FRAME** — the framework used to build the blockchain (like Ruby on Rails, but for blockchains)
- **Rust** — the programming language the blockchain is written in (fast and safe)
- **WebAssembly (WASM)** — the format the blockchain logic compiles to so it can run anywhere
- **Next.js** — the framework for the website (same tech used by major websites)
- **React Native** — the framework for the phone app (one codebase, runs on iPhone and Android)
- **SubQuery** — reads blockchain events and builds a GraphQL database for fast queries
- **IPFS** — decentralized file storage for event metadata (images, descriptions, dates)
- **Polkadot.js** — the library the website and app use to talk to the blockchain

---

## Project Status

BlocRoc is currently in the **scaffolding phase** — the structure, configuration, and skeleton logic are in place. The next steps are:

1. Connect the pallets to each other (e.g. buying a ticket on the marketplace should move the ticket ownership AND transfer the payment in one atomic step)
2. Build the website UI
3. Build the scanner app UI
4. Write benchmarks to calculate accurate transaction fees
5. Deploy a public testnet
