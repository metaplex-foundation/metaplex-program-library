import * as beet from '@metaplex-foundation/beet';
export type ApproveUseAuthorityArgs = {
  numberOfUses: beet.bignum;
};
export const approveUseAuthorityArgsStruct = new beet.BeetArgsStruct<ApproveUseAuthorityArgs>(
  [['numberOfUses', beet.u64]],
  'ApproveUseAuthorityArgs',
);
