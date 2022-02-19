import * as beet from '@metaplex-foundation/beet';
export type MintPrintingTokensViaTokenArgs = {
  supply: beet.bignum;
};
export const mintPrintingTokensViaTokenArgsStruct =
  new beet.BeetArgsStruct<MintPrintingTokensViaTokenArgs>(
    [['supply', beet.u64]],
    'MintPrintingTokensViaTokenArgs',
  );
