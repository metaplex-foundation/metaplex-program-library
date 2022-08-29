/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as web3 from '@solana/web3.js'
import * as beet from '@metaplex-foundation/beet'
import * as beetSolana from '@metaplex-foundation/beet-solana'
import {
  WhitelistTokenMode,
  whitelistTokenModeBeet,
} from './WhitelistTokenMode'
export type Whitelist = {
  mint: web3.PublicKey
  presale: boolean
  discountPrice: beet.COption<beet.bignum>
  mode: WhitelistTokenMode
}

/**
 * @category userTypes
 * @category generated
 */
export const whitelistBeet = new beet.FixableBeetArgsStruct<Whitelist>(
  [
    ['mint', beetSolana.publicKey],
    ['presale', beet.bool],
    ['discountPrice', beet.coption(beet.u64)],
    ['mode', whitelistTokenModeBeet],
  ],
  'Whitelist'
)