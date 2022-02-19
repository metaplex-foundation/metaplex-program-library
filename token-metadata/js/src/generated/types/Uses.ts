import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
export type Uses = {
  useMethod: definedTypes.UseMethod;
  remaining: beet.bignum;
  total: beet.bignum;
};
export const usesStruct = new beet.BeetArgsStruct<Uses>(
  [
    ['useMethod', definedTypes.useMethodStruct],
    ['remaining', beet.u64],
    ['total', beet.u64],
  ],
  'Uses',
);
