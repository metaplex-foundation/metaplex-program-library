import test from 'tape';
import {
  assertConfirmedTransaction,
  assertError,
  defaultSendOptions,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman';

import { Connection, Keypair, SYSVAR_CLOCK_PUBKEY, Transaction } from '@solana/web3.js';

import { createAndSignTransaction, logDebug } from '../utils';

import { createCloseMarketInstruction } from '../../src/instructions';

type CloseMarketParams = {
  test: test.Test;
  transactionHandler: PayerTransactionHandler;
  payer: Keypair;
  connection: Connection;
  market: Keypair;
};

export const closeMarket = async ({
  test,
  transactionHandler,
  payer,
  connection,
  market,
}: CloseMarketParams): Promise<void> => {
  const instruction = await createCloseMarketInstruction({
    market: market.publicKey,
    owner: payer.publicKey,
    clock: SYSVAR_CLOCK_PUBKEY,
  });

  const marketTx: Transaction = await createAndSignTransaction(
    connection,
    payer,
    [instruction],
    [payer],
  );

  const MarketRes = await transactionHandler.sendAndConfirmTransaction(
    marketTx,
    [payer],
    defaultSendOptions,
  );

  logDebug(`market: ${market.publicKey}`);
  assertConfirmedTransaction(test, MarketRes.txConfirmed);
};

export const closeMarketLimitedDuration = async ({
  test,
  transactionHandler,
  payer,
  connection,
  market,
}: CloseMarketParams): Promise<void> => {
  const instruction = await createCloseMarketInstruction({
    market: market.publicKey,
    owner: payer.publicKey,
    clock: SYSVAR_CLOCK_PUBKEY,
  });

  const marketTx: Transaction = await createAndSignTransaction(
    connection,
    payer,
    [instruction],
    [payer],
  );

  try {
    await transactionHandler.sendAndConfirmTransaction(marketTx, [payer], defaultSendOptions);

    test.fail('expected transaction to fail due to limited market duration ');
  } catch (error) {
    assertError(test, error, [/0x1782/i]);
  }

  logDebug(`market: ${market.publicKey}`);
};
