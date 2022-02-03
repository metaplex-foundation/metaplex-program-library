import * as beet from '@metaplex-foundation/beet';
export enum SellingResourceState {
  Uninitialized,
  Created,
  InUse,
  Exhausted,
  Stopped,
}
export const sellingResourceStateEnum = beet.fixedScalarEnum(
  SellingResourceState,
) as beet.FixedSizeBeet<SellingResourceState, SellingResourceState>;
