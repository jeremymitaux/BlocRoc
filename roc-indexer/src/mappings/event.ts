// Mapping handlers for pallet-event events.

import { SubstrateEvent } from '@subql/types';

export async function handleEventCreated(event: SubstrateEvent): Promise<void> {
  // event.event.data: [event_id, organizer, capacity]
  throw new Error('handleEventCreated: not yet implemented');
}

export async function handleEventCancelled(event: SubstrateEvent): Promise<void> {
  // event.event.data: [event_id]
  throw new Error('handleEventCancelled: not yet implemented');
}
