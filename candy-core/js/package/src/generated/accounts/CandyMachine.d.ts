/// <reference types="node" />
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';
import { CandyMachineData } from '../types/CandyMachineData';
export declare type CandyMachineArgs = {
    features: beet.bignum;
    wallet: web3.PublicKey;
    authority: web3.PublicKey;
    updateAuthority: web3.PublicKey;
    collectionMint: beet.COption<web3.PublicKey>;
    itemsRedeemed: beet.bignum;
    data: CandyMachineData;
};
export declare const candyMachineDiscriminator: number[];
export declare class CandyMachine implements CandyMachineArgs {
    readonly features: beet.bignum;
    readonly wallet: web3.PublicKey;
    readonly authority: web3.PublicKey;
    readonly updateAuthority: web3.PublicKey;
    readonly collectionMint: beet.COption<web3.PublicKey>;
    readonly itemsRedeemed: beet.bignum;
    readonly data: CandyMachineData;
    private constructor();
    static fromArgs(args: CandyMachineArgs): CandyMachine;
    static fromAccountInfo(accountInfo: web3.AccountInfo<Buffer>, offset?: number): [CandyMachine, number];
    static fromAccountAddress(connection: web3.Connection, address: web3.PublicKey): Promise<CandyMachine>;
    static gpaBuilder(programId?: web3.PublicKey): beetSolana.GpaBuilder<CandyMachineArgs & {
        accountDiscriminator: number[];
    }>;
    static deserialize(buf: Buffer, offset?: number): [CandyMachine, number];
    serialize(): [Buffer, number];
    static byteSize(args: CandyMachineArgs): number;
    static getMinimumBalanceForRentExemption(args: CandyMachineArgs, connection: web3.Connection, commitment?: web3.Commitment): Promise<number>;
    pretty(): {
        features: number | {
            toNumber: () => number;
        };
        wallet: string;
        authority: string;
        updateAuthority: string;
        collectionMint: web3.PublicKey;
        itemsRedeemed: number | {
            toNumber: () => number;
        };
        data: CandyMachineData;
    };
}
export declare const candyMachineBeet: beet.FixableBeetStruct<CandyMachine, CandyMachineArgs & {
    accountDiscriminator: number[];
}>;
