import { Program } from '@metaplex-foundation/mpl-core';
import { PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from './consts';

export class FixedPriceSaleProgram extends Program {
  static readonly PUBKEY = new PublicKey(PROGRAM_ID);
}
