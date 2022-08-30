import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import { WhitelistTokenMode } from './WhitelistTokenMode';
export declare type Whitelist = {
    mint: web3.PublicKey;
    presale: boolean;
    discountPrice: beet.COption<beet.bignum>;
    mode: WhitelistTokenMode;
};
export declare const whitelistBeet: beet.FixableBeetArgsStruct<Whitelist>;
