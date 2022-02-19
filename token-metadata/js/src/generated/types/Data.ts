import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
export type Data = {
  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number;
  creators: beet.COption<definedTypes.Creator[]>;
};
export const dataStruct = new beet.FixableBeetArgsStruct<Data>(
  [
    ['name', beet.utf8String],
    ['symbol', beet.utf8String],
    ['uri', beet.utf8String],
    ['sellerFeeBasisPoints', beet.u16],
    ['creators', beet.coption(beet.array(definedTypes.creatorStruct))],
  ],
  'Data',
);
