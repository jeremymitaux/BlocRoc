// Core scanner logic — QR decode → RPC call → display result
// Placeholder — logic to be implemented

export type ScanResult =
  | { status: 'valid'; ticketId: string; eventId: string; owner: string }
  | { status: 'already_used'; ticketId: string }
  | { status: 'invalid'; reason: string };

/**
 * Parse a raw QR code string into a ticket reference.
 * Expected QR format: `blocroc://ticket/<ticketId>`
 */
export function parseTicketQr(raw: string): { ticketId: string } | null {
  const match = raw.match(/^blocroc:\/\/ticket\/(\d+)$/);
  if (!match) return null;
  return { ticketId: match[1] };
}

/**
 * Submit a `scanner.validateEntry` extrinsic to the BlocRoc node and
 * return whether entry should be granted.
 *
 * Not yet implemented — will use `@polkadot/api`.
 */
export async function validateTicket(
  _ticketId: string,
  _eventId: string,
): Promise<ScanResult> {
  throw new Error('validateTicket: not yet implemented');
}
