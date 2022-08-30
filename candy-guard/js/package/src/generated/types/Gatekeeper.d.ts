import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
export declare type Gatekeeper = {
    gatekeeperNetwork: web3.PublicKey;
    expireOnUse: boolean;
};
export declare const gatekeeperBeet: beet.BeetArgsStruct<Gatekeeper>;
