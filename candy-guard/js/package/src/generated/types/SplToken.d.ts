import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
export declare type SplToken = {
    amount: beet.bignum;
    tokenMint: web3.PublicKey;
};
export declare const splTokenBeet: beet.BeetArgsStruct<SplToken>;
