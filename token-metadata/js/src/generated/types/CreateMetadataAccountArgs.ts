import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
export type CreateMetadataAccountArgs = {
  data: definedTypes.Data;
  isMutable: boolean;
};
export const createMetadataAccountArgsStruct = new beet.BeetArgsStruct<CreateMetadataAccountArgs>(
  [
    ['data', definedTypes.dataStruct],
    ['isMutable', beet.bool],
  ],
  'CreateMetadataAccountArgs',
);
