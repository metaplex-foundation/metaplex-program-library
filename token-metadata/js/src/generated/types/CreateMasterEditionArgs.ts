import * as beet from '@metaplex-foundation/beet';
export type CreateMasterEditionArgs = {
  maxSupply: beet.COption<beet.bignum>;
};
export const createMasterEditionArgsStruct =
  new beet.FixableBeetArgsStruct<CreateMasterEditionArgs>(
    [['maxSupply', beet.coption(beet.u64)]],
    'CreateMasterEditionArgs',
  );
