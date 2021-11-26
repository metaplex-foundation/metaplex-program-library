import { PublicKey } from '@solana/web3.js';

export type ParamsWithStore<P> = P & { store: PublicKey };
