// Mapping handlers for pallet-scanner events.

import { SubstrateEvent } from '@subql/types';

export async function handleEntryValidated(event: SubstrateEvent): Promise<void> {
  // event.event.data: [event_id, ticket_id, scanner, block]
  throw new Error('handleEntryValidated: not yet implemented');
}
