import { clusterApiUrl } from '@solana/web3.js';
import { inspect } from 'util';
import debug from 'debug';
import test from 'tape';
import { LOCALHOST } from '@metaplex-foundation/amman';

export * from './address-labels';
export * from './metadata';

export const logError = debug('mpl:tm-test:error');
export const logInfo = debug('mpl:tm-test:info');
export const logDebug = debug('mpl:tm-test:debug');
export const logTrace = debug('mpl:tm-test:trace');

export const programIds = {
  metadata: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
  vault: 'vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn',
  auction: 'auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8',
  metaplex: 'p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98',
};

export const DEVNET = clusterApiUrl('devnet');
export const connectionURL = process.env.USE_DEVNET != null ? DEVNET : LOCALHOST;

export function dump(x: object) {
  console.log(inspect(x, { depth: 5 }));
}

export function killStuckProcess() {
  // solana web socket keeps process alive for longer than necessary which we
  // "fix" here
  test.onFinish(() => process.exit(0));
}
