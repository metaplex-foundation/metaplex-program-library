export * from './CollectionAuthorityRecord';
export * from './Edition';
export * from './EditionMarker';
export * from './MasterEditionV1';
export * from './MasterEditionV2';
export * from './Metadata';
export * from './MetadataDelegateRecord';
export * from './ReservationListV1';
export * from './ReservationListV2';
export * from './TokenOwnedEscrow';
export * from './TokenRecord';
export * from './UseAuthorityRecord';

import { CollectionAuthorityRecord } from './CollectionAuthorityRecord';
import { MetadataDelegateRecord } from './MetadataDelegateRecord';
import { Edition } from './Edition';
import { EditionMarker } from './EditionMarker';
import { TokenOwnedEscrow } from './TokenOwnedEscrow';
import { MasterEditionV2 } from './MasterEditionV2';
import { MasterEditionV1 } from './MasterEditionV1';
import { Metadata } from './Metadata';
import { TokenRecord } from './TokenRecord';
import { ReservationListV2 } from './ReservationListV2';
import { ReservationListV1 } from './ReservationListV1';
import { UseAuthorityRecord } from './UseAuthorityRecord';

export const accountProviders = {
  CollectionAuthorityRecord,
  MetadataDelegateRecord,
  Edition,
  EditionMarker,
  TokenOwnedEscrow,
  MasterEditionV2,
  MasterEditionV1,
  Metadata,
  TokenRecord,
  ReservationListV2,
  ReservationListV1,
  UseAuthorityRecord,
};
