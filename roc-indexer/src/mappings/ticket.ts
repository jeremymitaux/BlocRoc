// Mapping handlers for pallet-ticket events.
// Logic to be implemented — stubs show the expected signature.

import { SubstrateEvent } from '@subql/types';
import { Ticket, TicketTransfer } from '../types';

export async function handleTicketMinted(event: SubstrateEvent): Promise<void> {
  // event.event.data: [ticket_id, event_id, owner]
  throw new Error('handleTicketMinted: not yet implemented');
}

export async function handleTicketTransferred(event: SubstrateEvent): Promise<void> {
  // event.event.data: [ticket_id, from, to]
  throw new Error('handleTicketTransferred: not yet implemented');
}

export async function handleTicketBurned(event: SubstrateEvent): Promise<void> {
  // event.event.data: [ticket_id, owner]
  throw new Error('handleTicketBurned: not yet implemented');
}

export async function handleTicketInvalidated(event: SubstrateEvent): Promise<void> {
  // event.event.data: [ticket_id]
  throw new Error('handleTicketInvalidated: not yet implemented');
}
