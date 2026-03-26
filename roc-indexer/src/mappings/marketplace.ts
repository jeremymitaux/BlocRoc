// Mapping handlers for pallet-marketplace events.

import { SubstrateEvent } from '@subql/types';

export async function handleTicketListed(event: SubstrateEvent): Promise<void> {
  // event.event.data: [listing_id, ticket_id, seller, price]
  throw new Error('handleTicketListed: not yet implemented');
}

export async function handleListingCancelled(event: SubstrateEvent): Promise<void> {
  // event.event.data: [listing_id, ticket_id]
  throw new Error('handleListingCancelled: not yet implemented');
}

export async function handleTicketPurchased(event: SubstrateEvent): Promise<void> {
  // event.event.data: [listing_id, ticket_id, seller, buyer, price]
  throw new Error('handleTicketPurchased: not yet implemented');
}
