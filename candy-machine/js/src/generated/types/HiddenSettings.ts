import * as beet from '@metaplex-foundation/beet';
export type HiddenSettings = {
  name: string;
  uri: string;
  hash: number[] /* size: 32 */;
};
export const hiddenSettingsStruct = new beet.FixableBeetArgsStruct<HiddenSettings>(
  [
    ['name', beet.utf8String],
    ['uri', beet.utf8String],
    ['hash', beet.uniformFixedSizeArray(beet.u8, 32)],
  ],
  'HiddenSettings',
);
