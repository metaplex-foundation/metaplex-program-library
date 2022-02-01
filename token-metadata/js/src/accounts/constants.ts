export enum MetadataKey {
  Uninitialized = 0,
  MetadataV1 = 4,
  EditionV1 = 1,
  MasterEditionV1 = 2,
  MasterEditionV2 = 6,
  EditionMarker = 7,
  UseAuthorityRecord = 8,
  CollectionAuthorityRecord = 9,
}

export enum UseMethod {
  Burn = 0,
  Single = 1,
  Multiple = 2,
}

export enum TokenStandard {
  NonFungible = 0, // This is a master edition
  FungibleAsset = 1, // A token with metadata that can also have attributes
  Fungible = 2, // A token with simple metadata
  NonFungibleEdition = 3, // This is a limited edition
}
