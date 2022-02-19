import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
export type CreateMetadataAccountArgsV2 = {
  data: definedTypes.DataV2;
  isMutable: boolean;
};
export const createMetadataAccountArgsV2Struct =
  new beet.BeetArgsStruct<CreateMetadataAccountArgsV2>(
    [
      ['data', definedTypes.dataV2Struct],
      ['isMutable', beet.bool],
    ],
    'CreateMetadataAccountArgsV2',
  );
