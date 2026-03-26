// Polkadot.js API singleton for connecting to the BlocRoc node.
// Logic to be implemented.

import { ApiPromise, WsProvider } from '@polkadot/api';

const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL ?? 'ws://127.0.0.1:9944';

let _api: ApiPromise | null = null;

export async function getApi(): Promise<ApiPromise> {
  if (_api && _api.isConnected) return _api;

  const provider = new WsProvider(RPC_URL);
  _api = await ApiPromise.create({ provider });
  return _api;
}
