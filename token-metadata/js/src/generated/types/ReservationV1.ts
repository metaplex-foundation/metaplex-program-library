import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';
import * as beet from '@metaplex-foundation/beet';
export type ReservationV1 = {
  address: web3.PublicKey;
  spotsRemaining: number;
  totalSpots: number;
};
export const reservationV1Struct = new beet.BeetArgsStruct<ReservationV1>(
  [
    ['address', beetSolana.publicKey],
    ['spotsRemaining', beet.u8],
    ['totalSpots', beet.u8],
  ],
  'ReservationV1',
);
