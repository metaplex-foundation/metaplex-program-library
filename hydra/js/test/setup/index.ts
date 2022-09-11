/* eslint-disable @typescript-eslint/no-explicit-any */
import test from 'tape';

export * from './log';
export * from './amman';

export function killStuckProcess() {
  test.onFinish(() => process.exit(0));
}

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
