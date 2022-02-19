import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
export type DataV2 = {
  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number;
  creators: beet.COption<definedTypes.Creator[]>;
  collection: beet.COption<definedTypes.Collection>;
  uses: beet.COption<definedTypes.Uses>;
};
export const dataV2Struct = new beet.FixableBeetArgsStruct<DataV2>(
  [
    ['name', beet.utf8String],
    ['symbol', beet.utf8String],
    ['uri', beet.utf8String],
    ['sellerFeeBasisPoints', beet.u16],
    ['creators', beet.coption(beet.array(definedTypes.creatorStruct))],
    ['collection', beet.coption(definedTypes.collectionStruct)],
    ['uses', beet.coption(definedTypes.usesStruct)],
  ],
  'DataV2',
);
