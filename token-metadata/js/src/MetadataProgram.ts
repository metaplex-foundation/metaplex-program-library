import { PublicKey } from '@solana/web3.js';
import { Program, config } from '@metaplex/mpl-core';

export enum MetadataKey {
  Uninitialized = 0,
  MetadataV1 = 4,
  EditionV1 = 1,
  MasterEditionV1 = 2,
  MasterEditionV2 = 6,
  EditionMarker = 7,
}

export class MetadataProgram extends Program {
  static readonly PREFIX = 'metadata';
  static readonly PUBKEY = new PublicKey(config.programs.metadata);
}
