import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';
import * as beet from '@metaplex-foundation/beet';
export type GatekeeperConfig = {
  gatekeeperNetwork: web3.PublicKey;
  expireOnUse: boolean;
};
export const gatekeeperConfigStruct = new beet.BeetArgsStruct<GatekeeperConfig>(
  [
    ['gatekeeperNetwork', beetSolana.publicKey],
    ['expireOnUse', beet.bool],
  ],
  'GatekeeperConfig',
);
