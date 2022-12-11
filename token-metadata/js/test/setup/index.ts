import test from 'tape';

export * from './amman';
export * from './txs-init';
export * from './log';

export function killStuckProcess() {
  test.onFinish(() => process.exit(0));
}

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
