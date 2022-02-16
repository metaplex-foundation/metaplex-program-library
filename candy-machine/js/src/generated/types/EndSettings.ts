import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
export type EndSettings = {
  endSettingType: definedTypes.EndSettingType;
  number: beet.bignum;
};
export const endSettingsStruct = new beet.BeetArgsStruct<EndSettings>(
  [
    ['endSettingType', definedTypes.endSettingTypeStruct],
    ['number', beet.u64],
  ],
  'EndSettings',
);
