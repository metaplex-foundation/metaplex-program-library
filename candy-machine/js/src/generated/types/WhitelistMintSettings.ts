import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';
export type WhitelistMintSettings = {
  mode: definedTypes.WhitelistMintMode;
  mint: web3.PublicKey;
  presale: boolean;
  discountPrice: beet.COption<beet.bignum>;
};
export const whitelistMintSettingsStruct = new beet.FixableBeetArgsStruct<WhitelistMintSettings>(
  [
    ['mode', definedTypes.whitelistMintModeStruct],
    ['mint', beetSolana.publicKey],
    ['presale', beet.bool],
    ['discountPrice', beet.coption(beet.u64)],
  ],
  'WhitelistMintSettings',
);
