import { Program } from '@metaplex-foundation/mpl-core';
import { PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from './consts';

export class MembershipTokenProgram extends Program {
  static readonly PUBKEY = new PublicKey(PROGRAM_ID);
}
