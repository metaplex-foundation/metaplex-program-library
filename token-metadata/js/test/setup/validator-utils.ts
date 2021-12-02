import { Connection, Keypair, SystemProgram, Transaction } from '@solana/web3.js';
import { airdrop, connectionURL, logDebug } from 'test/utils';
import waitOn from 'wait-on';

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/*
 * Right after a local test validator is started up it doesn't seem to charge
 * fees at times.
 * Here we make sure we don't consider it started up until it does charge fees.
 */
export async function ensureValidatorIsUpAndChargesFees(connectionURL: string) {
  logDebug('Waiting for validator to come up ...');
  await waitOn({
    resources: [connectionURL],
    interval: 1000,
    validateStatus: (status) => status === 405,
    log: false,
  });
  logDebug('Ensuring validator charges fees ...');

  const payer = Keypair.generate();
  const connection = new Connection(connectionURL, 'confirmed');
  await airdrop(connection, payer.publicKey, 200);

  return ensureFees(connectionURL, payer);
}

async function ensureFees(connectionURL: string, payer: Keypair) {
  const receiver = Keypair.generate();
  const connection = new Connection(connectionURL, 'confirmed');
  const transferIx = SystemProgram.transfer({
    lamports: 1000,
    fromPubkey: payer.publicKey,
    toPubkey: receiver.publicKey,
  });
  const transaction = new Transaction().add(transferIx);
  const recentBlockhash = (await connection.getRecentBlockhash('confirmed')).blockhash;
  transaction.recentBlockhash = recentBlockhash;
  const sig = await connection.sendTransaction(transaction, [payer]);
  await connection.confirmTransaction(sig);
  const confirmedTx = await connection.getConfirmedTransaction(sig);

  if (confirmedTx.meta.fee === 0) {
    await sleep(2000);
    logDebug('Transaction completed without charging fees, trying again ...');
    return ensureFees(connectionURL, payer);
  }
  return confirmedTx;
}
