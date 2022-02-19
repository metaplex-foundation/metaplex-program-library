import * as beet from '@metaplex-foundation/beet';
export enum Key {
  Uninitialized,
  EditionV1,
  MasterEditionV1,
  ReservationListV1,
  MetadataV1,
  ReservationListV2,
  MasterEditionV2,
  EditionMarker,
  UseAuthorityRecord,
  CollectionAuthorityRecord,
}
export const keyEnum = beet.fixedScalarEnum(Key) as beet.FixedSizeBeet<Key, Key>;
