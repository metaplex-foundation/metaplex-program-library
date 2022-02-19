import * as beet from '@metaplex-foundation/beet';
export enum UseMethod {
  Burn,
  Multiple,
  Single,
}
export const useMethodEnum = beet.fixedScalarEnum(UseMethod) as beet.FixedSizeBeet<
  UseMethod,
  UseMethod
>;
