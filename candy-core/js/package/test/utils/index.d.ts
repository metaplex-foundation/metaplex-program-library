export * from './asserts';
export * from './constants';
export * from './errors';
import { Keypair, PublicKey } from '@solana/web3.js';
import { CandyMachineData } from 'src/generated';
export declare function getCandyMachinePDA(programId: PublicKey, base: Keypair): Promise<PublicKey>;
export declare function getCandyMachineSpace(data: CandyMachineData): number;
