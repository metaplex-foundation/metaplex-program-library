import { inspect } from 'util';
import test from 'tape';

export * from './address-labels';
export * from './consts';
export * from './log';
export * from './metadata';

export function dump(x: object) {
  console.log(inspect(x, { depth: 5 }));
}

export function killStuckProcess() {
  // solana web socket keeps process alive for longer than necessary which we
  // "fix" here
  test.onFinish(() => process.exit(0));
}
