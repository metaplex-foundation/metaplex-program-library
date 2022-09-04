import { config, Program } from '@metaplex-foundation/mpl-core';
import { PublicKey } from '@solana/web3.js';

export class MetadataProgram extends Program {
  static readonly PUBKEY = new PublicKey(config.programs.metadata);
}