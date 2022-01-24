import { clusterApiUrl, Signer, TransactionInstruction } from '@solana/web3.js';
import debug from 'debug';
import test from 'tape';
import { LOCALHOST } from '@metaplex-foundation/amman';
import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';

export * from './address-labels';

export const logError = debug('mpl:tm-test:error');
export const logInfo = debug('mpl:tm-test:info');
export const logDebug = debug('mpl:tm-test:debug');
export const logTrace = debug('mpl:tm-test:trace');

export const DEVNET = clusterApiUrl('devnet');
export const connectionURL = process.env.USE_DEVNET != null ? DEVNET : LOCALHOST;

export function killStuckProcess() {
  // solana web socket keeps process alive for longer than necessary which we
  // "fix" here
  test.onFinish(() => process.exit(0));
}

export async function createAndSignTransaction(
  instruction: TransactionInstruction,
  connection: Connection,
  payer: Keypair,
  signers: Signer[],
): Promise<Transaction> {
  const marketTx = new Transaction();
  marketTx.add(instruction);
  marketTx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  marketTx.feePayer = payer.publicKey;
  marketTx.partialSign(...signers);

  return marketTx;
}
