import * as beet from '@metaplex-foundation/beet';
export enum TokenStandard {
  NonFungible,
  FungibleAsset,
  Fungible,
  NonFungibleEdition,
}
export const tokenStandardEnum = beet.fixedScalarEnum(TokenStandard) as beet.FixedSizeBeet<
  TokenStandard,
  TokenStandard
>;
