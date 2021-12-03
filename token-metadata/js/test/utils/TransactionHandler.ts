import {
  ConfirmedTransaction,
  Connection,
  Keypair,
  PublicKey,
  RpcResponseAndContext,
  SendOptions,
  SignatureResult,
  Signer,
  Transaction,
  TransactionError,
  TransactionSignature,
} from '@solana/web3.js';
import { defaultSendOptions } from '.';

export type SendTransaction = (
  connection: Connection,
  transaction: Transaction,
  signers: Array<Signer>,
  options?: SendOptions,
) => Promise<TransactionSignature>;

export type TransactionSummary = {
  logMessages: string[];
  fee: number | undefined;
  slot: number;
  blockTime: number;
  err: TransactionError | null | undefined;
};

export type ConfirmedTransactionDetails = {
  txSignature: string;
  txRpcResponse: RpcResponseAndContext<SignatureResult>;
  txConfirmed: ConfirmedTransaction;
  txSummary: TransactionSummary;
};

function transactionSummary(tx: ConfirmedTransaction): TransactionSummary {
  const logMessages = tx.meta?.logMessages ?? [];
  const fee = tx.meta?.fee;
  const slot = tx.slot;
  const blockTime = tx.blockTime ?? 0;
  const err = tx.meta?.err;
  return { logMessages, fee, slot, blockTime, err };
}

/*
 * Using an interface here in order to support wallet based transaction handler
 * in the future
 */
export type TransactionHandler = {
  publicKey: PublicKey;

  sendAndConfirmTransaction(
    transaction: Transaction,
    signers: Array<Signer>,
    options?: SendOptions,
  ): Promise<ConfirmedTransactionDetails>;
};

export class PayerTransactionHandler implements TransactionHandler {
  constructor(private readonly connection: Connection, private readonly payer: Keypair) {}

  get publicKey() {
    return this.payer.publicKey;
  }

  async sendAndConfirmTransaction(
    transaction: Transaction,
    signers: Array<Signer>,
    options?: SendOptions,
  ): Promise<ConfirmedTransactionDetails> {
    transaction.recentBlockhash = (await this.connection.getRecentBlockhash()).blockhash;

    const txSignature = await this.connection.sendTransaction(
      transaction,
      [this.payer, ...signers],
      options ?? defaultSendOptions,
    );
    const txRpcResponse = await this.connection.confirmTransaction(txSignature);
    const txConfirmed = await this.connection.getConfirmedTransaction(txSignature);
    const txSummary = transactionSummary(txConfirmed);

    return { txSignature, txRpcResponse, txConfirmed, txSummary };
  }
}
