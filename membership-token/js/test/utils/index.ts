import debug from 'debug';
import test from 'tape';
import { clusterApiUrl } from '@solana/web3.js';
import { LOCALHOST } from '@metaplex-foundation/amman';

export * from './address-labels';
export { createAndSignTransaction } from './createAndSignTx';

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
