import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';
export type Reservation = {
  address: web3.PublicKey;
  spotsRemaining: beet.bignum;
  totalSpots: beet.bignum;
};
export const reservationStruct = new beet.BeetArgsStruct<Reservation>(
  [
    ['address', beetSolana.publicKey],
    ['spotsRemaining', beet.u64],
    ['totalSpots', beet.u64],
  ],
  'Reservation',
);
