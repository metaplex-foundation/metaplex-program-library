/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
export type ConfigLineSettings = {
  prefixName: string;
  nameLength: number;
  prefixUri: string;
  uriLength: number;
  isSequential: boolean;
};

/**
 * @category userTypes
 * @category generated
 */
export const configLineSettingsBeet = new beet.FixableBeetArgsStruct<ConfigLineSettings>(
  [
    ['prefixName', beet.utf8String],
    ['nameLength', beet.u32],
    ['prefixUri', beet.utf8String],
    ['uriLength', beet.u32],
    ['isSequential', beet.bool],
  ],
  'ConfigLineSettings',
);
