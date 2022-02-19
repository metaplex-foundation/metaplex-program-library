import * as beet from '@metaplex-foundation/beet';
export type MintNewEditionFromMasterEditionViaTokenArgs = {
  edition: beet.bignum;
};
export const mintNewEditionFromMasterEditionViaTokenArgsStruct =
  new beet.BeetArgsStruct<MintNewEditionFromMasterEditionViaTokenArgs>(
    [['edition', beet.u64]],
    'MintNewEditionFromMasterEditionViaTokenArgs',
  );
