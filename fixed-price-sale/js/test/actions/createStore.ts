import test from 'tape';
import { PayerTransactionHandler } from '@metaplex-foundation/amman-client';
import { Connection, Keypair } from '@solana/web3.js';

import {
  createCreateStoreInstruction,
  CreateStoreInstructionArgs,
} from '../../src/generated/instructions';

import { createAndSignTransaction, logDebug } from '../utils';

type CreateStoreParams = {
  test: test.Test;
  transactionHandler: PayerTransactionHandler;
  payer: Keypair;
  connection: Connection;
  params: CreateStoreInstructionArgs;
};

export const createStore = async ({
  test,
  transactionHandler,
  payer,
  connection,
  params,
}: CreateStoreParams): Promise<Keypair> => {
  const store = Keypair.generate();

  const instruction = createCreateStoreInstruction(
    {
      store: store.publicKey,
      admin: payer.publicKey,
    },
    params,
  );

  const transaction = await createAndSignTransaction(connection, payer, [instruction], [store]);

  await transactionHandler.sendAndConfirmTransaction(transaction, [store]).assertSuccess(test);
  logDebug(`store: ${store.publicKey}`);

  return store;
};
