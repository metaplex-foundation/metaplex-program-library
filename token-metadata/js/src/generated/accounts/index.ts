export * from './CollectionAuthorityRecord';
export * from './Edition';
export * from './EditionMarker';
export * from './MasterEditionV1';
export * from './MasterEditionV2';
export * from './Metadata';
export * from './ReservationListV1';
export * from './ReservationListV2';
export * from './TokenOwnedEscrow';
export * from './UseAuthorityRecord';

import { UseAuthorityRecord } from './UseAuthorityRecord';
import { CollectionAuthorityRecord } from './CollectionAuthorityRecord';
import { Metadata } from './Metadata';
import { MasterEditionV2 } from './MasterEditionV2';
import { MasterEditionV1 } from './MasterEditionV1';
import { Edition } from './Edition';
import { ReservationListV2 } from './ReservationListV2';
import { ReservationListV1 } from './ReservationListV1';
import { EditionMarker } from './EditionMarker';
import { TokenOwnedEscrow } from './TokenOwnedEscrow';

export const accountProviders = {
  UseAuthorityRecord,
  CollectionAuthorityRecord,
  Metadata,
  MasterEditionV2,
  MasterEditionV1,
  Edition,
  ReservationListV2,
  ReservationListV1,
  EditionMarker,
  TokenOwnedEscrow,
};
