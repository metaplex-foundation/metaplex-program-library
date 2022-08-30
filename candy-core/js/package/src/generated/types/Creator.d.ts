import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
export declare type Creator = {
    address: web3.PublicKey;
    verified: boolean;
    percentageShare: number;
};
export declare const creatorBeet: beet.BeetArgsStruct<Creator>;
