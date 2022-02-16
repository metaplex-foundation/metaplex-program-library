import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';
import * as beet from '@metaplex-foundation/beet';
export type Creator = {
  address: web3.PublicKey;
  verified: boolean;
  share: number;
};
export const creatorBeet = new beet.BeetArgsStruct<Creator>(
  [
    ['address', beetSolana.publicKey],
    ['verified', beet.bool],
    ['share', beet.u8],
  ],
  'Creator',
);
