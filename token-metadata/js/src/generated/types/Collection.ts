import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';
export type Collection = {
  verified: boolean;
  key: web3.PublicKey;
};
export const collectionStruct = new beet.BeetArgsStruct<Collection>(
  [
    ['verified', beet.bool],
    ['key', beetSolana.publicKey],
  ],
  'Collection',
);
