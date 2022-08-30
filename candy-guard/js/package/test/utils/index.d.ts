export * from './asserts';
export * from './constants';
export * from './errors';
import { Keypair, PublicKey } from '@solana/web3.js';
export declare function getCandyGuardPDA(programId: PublicKey, base: Keypair): Promise<PublicKey>;
