import { ConfirmedTransactionAssertablePromise, GenLabeledKeypair, LoadOrGenKeypair, PayerTransactionHandler } from '@metaplex-foundation/amman-client';
import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import { Test } from 'tape';
import * as program from '../../src/generated';
import { CandyMachineData } from '../../src/generated';
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
    minter(): Promise<{
        fstTxHandler: PayerTransactionHandler;
        connection: Connection;
        minter: PublicKey;
        minterPair: Keypair;
    }>;
    create(t: Test, payer: Keypair, data: program.CandyMachineData, handler: PayerTransactionHandler, connection: Connection): Promise<{
        tx: ConfirmedTransactionAssertablePromise;
        candyMachine: PublicKey;
    }>;
    addConfigLines(t: Test, candyMachine: PublicKey, payer: Keypair, lines: program.ConfigLine[], handler: PayerTransactionHandler): Promise<{
        txs: Transaction[];
    }>;
    updateCandyMachine(t: Test, candyMachine: PublicKey, wallet: PublicKey, payer: Keypair, data: CandyMachineData, handler: PayerTransactionHandler): Promise<{
        tx: ConfirmedTransactionAssertablePromise;
    }>;
    mint(t: Test, candyMachine: PublicKey, payer: Keypair, handler: PayerTransactionHandler, connection: Connection): Promise<{
        tx: ConfirmedTransactionAssertablePromise;
    }>;
    withdraw(t: Test, candyMachine: PublicKey, payer: Keypair, handler: PayerTransactionHandler): Promise<{
        tx: ConfirmedTransactionAssertablePromise;
    }>;
}
