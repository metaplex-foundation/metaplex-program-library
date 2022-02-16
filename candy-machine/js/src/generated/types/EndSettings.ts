import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
export type EndSettings = {
  endSettingType: definedTypes.EndSettingType;
  number: beet.bignum;
};
export const endSettingsBeet = new beet.BeetArgsStruct<EndSettings>(
  [
    ['endSettingType', definedTypes.endSettingTypeBeet],
    ['number', beet.u64],
  ],
  'EndSettings',
);
