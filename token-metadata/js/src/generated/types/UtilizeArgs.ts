import * as beet from '@metaplex-foundation/beet';
export type UtilizeArgs = {
  numberOfUses: beet.bignum;
};
export const utilizeArgsStruct = new beet.BeetArgsStruct<UtilizeArgs>(
  [['numberOfUses', beet.u64]],
  'UtilizeArgs',
);
