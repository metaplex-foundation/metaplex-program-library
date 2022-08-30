import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
export declare type ThirdPartySigner = {
    signerKey: web3.PublicKey;
};
export declare const thirdPartySignerBeet: beet.BeetArgsStruct<ThirdPartySigner>;
