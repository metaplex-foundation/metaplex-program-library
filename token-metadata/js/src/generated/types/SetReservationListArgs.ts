import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
export type SetReservationListArgs = {
  reservations: definedTypes.Reservation[];
  totalReservationSpots: beet.COption<beet.bignum>;
  offset: beet.bignum;
  totalSpotOffset: beet.bignum;
};
export const setReservationListArgsStruct = new beet.FixableBeetArgsStruct<SetReservationListArgs>(
  [
    ['reservations', beet.array(definedTypes.reservationStruct)],
    ['totalReservationSpots', beet.coption(beet.u64)],
    ['offset', beet.u64],
    ['totalSpotOffset', beet.u64],
  ],
  'SetReservationListArgs',
);
