import { ConfirmedTransactionAssertablePromise, GenLabeledKeypair, LoadOrGenKeypair, PayerTransactionHandler } from '@metaplex-foundation/amman-client';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { Test } from 'tape';
import { CandyGuardData } from '../../src/generated';
export declare class InitTransactions {
    readonly resuseKeypairs: boolean;
    readonly getKeypair: LoadOrGenKeypair | GenLabeledKeypair;
    constructor(resuseKeypairs?: boolean);
    payer(): Promise<{
        fstTxHandler: PayerTransactionHandler;
        connection: Connection;
        payer: PublicKey;
        payerPair: Keypair;
    }>;
    authority(): Promise<{
        fstTxHandler: PayerTransactionHandler;
        connection: Connection;
        authority: PublicKey;
        authorityPair: Keypair;
    }>;
    initialize(t: Test, data: CandyGuardData, payer: Keypair, handler: PayerTransactionHandler): Promise<{
        tx: ConfirmedTransactionAssertablePromise;
        candyGuard: PublicKey;
    }>;
    update(t: Test, candyGuard: PublicKey, data: CandyGuardData, payer: Keypair, handler: PayerTransactionHandler): Promise<{
        tx: ConfirmedTransactionAssertablePromise;
    }>;
}
