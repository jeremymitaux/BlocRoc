# BlocRoc API Specification

## On-Chain RPC (Polkadot.js / JSON-RPC)

Connect to: `ws://localhost:9944` (dev) | `wss://rpc.blocroc.io` (mainnet)

### Extrinsics

All extrinsics must be signed and submitted via `api.tx.<pallet>.<method>`.

---

#### `ticket.mint(event_id, owner)`

Mint a ticket for an event.

| Parameter | Type | Description |
|---|---|---|
| `event_id` | `u64` | The event to mint a ticket for |
| `owner` | `AccountId` | Account to receive the ticket |

Emits: `ticket.TicketMinted { ticket_id, event_id, owner }`

---

#### `ticket.transfer(ticket_id, to)`

Transfer a ticket to another account.

| Parameter | Type | Description |
|---|---|---|
| `ticket_id` | `u64` | Ticket to transfer |
| `to` | `AccountId` | Recipient |

Errors: `TicketNotFound`, `NotTicketOwner`, `TicketAlreadyUsed`

---

#### `ticket.burn(ticket_id)`

Destroy a ticket (e.g. refund flow).

Errors: `TicketNotFound`, `NotTicketOwner`

---

#### `ticket.invalidate(ticket_id)`

Mark a ticket as used for entry.

Errors: `TicketNotFound`, `TicketAlreadyUsed`

---

#### `event.createEvent(capacity, metadata_cid)`

Create a new event.

| Parameter | Type | Description |
|---|---|---|
| `capacity` | `u32` | Maximum tickets |
| `metadata_cid` | `BoundedVec<u8, 64>` | IPFS CID of event metadata |

Emits: `event.EventCreated { event_id, organizer, capacity }`

---

#### `event.cancelEvent(event_id)`

Cancel an event.

Errors: `EventNotFound`, `NotOrganizer`, `EventCancelled`

---

#### `marketplace.list(ticket_id, price)`

List a ticket for sale.

| Parameter | Type | Description |
|---|---|---|
| `ticket_id` | `u64` | Ticket to list |
| `price` | `u128` | Asking price (ROC, smallest unit) |

Errors: `AlreadyListed`

---

#### `marketplace.delist(listing_id)`

Cancel a listing.

Errors: `ListingNotFound`, `NotSeller`

---

#### `marketplace.buy(listing_id)`

Purchase a listed ticket.

Errors: `ListingNotFound`

---

#### `scanner.authorizeScanner(event_id, scanner)`

Authorize a scanner account for an event.

---

#### `scanner.validateEntry(event_id, ticket_id)`

Validate a ticket for entry. Caller must be an authorized scanner.

Errors: `UnauthorizedScanner`, `AlreadyScanned`

---

## Off-Chain GraphQL (SubQuery)

Endpoint: `http://localhost:3001` (dev) | `https://indexer.blocroc.io` (mainnet)

### Example Queries

```graphql
# List all active events
query {
  rocEvents(filter: { cancelled: { equalTo: false } }) {
    nodes {
      id
      organizer
      capacity
      sold
      metadataCid
      createdAt
    }
  }
}

# Get all tickets owned by an address
query ($owner: String!) {
  tickets(filter: { owner: { equalTo: $owner }, burned: { equalTo: false } }) {
    nodes {
      id
      eventId
      used
      mintedAt
    }
  }
}

# Get active marketplace listings for an event
query ($eventId: String!) {
  listings(filter: {
    active: { equalTo: true }
  }) {
    nodes {
      id
      ticketId
      seller
      price
      listedAt
    }
  }
}

# Get entry scan records for an event
query ($eventId: String!) {
  scanRecords(filter: { eventId: { equalTo: $eventId } }) {
    nodes {
      ticketId
      scanner
      blockNumber
      timestamp
    }
  }
}
```
