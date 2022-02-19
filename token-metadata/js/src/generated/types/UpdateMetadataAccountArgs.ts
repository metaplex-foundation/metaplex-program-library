import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';
export type UpdateMetadataAccountArgs = {
  data: beet.COption<definedTypes.Data>;
  updateAuthority: beet.COption<web3.PublicKey>;
  primarySaleHappened: beet.COption<boolean>;
};
export const updateMetadataAccountArgsStruct =
  new beet.FixableBeetArgsStruct<UpdateMetadataAccountArgs>(
    [
      ['data', beet.coption(definedTypes.dataStruct)],
      ['updateAuthority', beet.coption(beetSolana.publicKey)],
      ['primarySaleHappened', beet.coption(beet.bool)],
    ],
    'UpdateMetadataAccountArgs',
  );
