import { Keypair, PublicKey, Signer, TransactionInstruction } from '@solana/web3.js';

export type InstructionsWithAccounts<T extends Record<string, PublicKey | Keypair>> = [
  TransactionInstruction[],
  Signer[],
  T,
];
