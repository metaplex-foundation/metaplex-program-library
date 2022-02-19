import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';
export type UpdateMetadataAccountArgsV2 = {
  data: beet.COption<definedTypes.DataV2>;
  updateAuthority: beet.COption<web3.PublicKey>;
  primarySaleHappened: beet.COption<boolean>;
  isMutable: beet.COption<boolean>;
};
export const updateMetadataAccountArgsV2Struct =
  new beet.FixableBeetArgsStruct<UpdateMetadataAccountArgsV2>(
    [
      ['data', beet.coption(definedTypes.dataV2Struct)],
      ['updateAuthority', beet.coption(beetSolana.publicKey)],
      ['primarySaleHappened', beet.coption(beet.bool)],
      ['isMutable', beet.coption(beet.bool)],
    ],
    'UpdateMetadataAccountArgsV2',
  );
