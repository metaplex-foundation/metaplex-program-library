import test from 'tape';
import { Connection, Keypair, Transaction } from '@solana/web3.js';
import { connectionURL, killStuckProcess } from './utils';
import {
  airdrop,
  assertConfirmedTransaction,
  PayerTransactionHandler,
  defaultSendOptions,
} from '@metaplex-foundation/amman';

import { addLabel } from './utils/address-labels';
import { createCreateStoreInstruction } from '../src/mpl-membership-token';

killStuckProcess();

test('create-store: success', async (t) => {
  const payer = Keypair.generate();
  const store = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const instruction = createCreateStoreInstruction(
    {
      store: store.publicKey,
      admin: payer.publicKey,
    },
    {
      name: 'izd5Pr9ltIAJL4ac8cYMUDlakSXNPnJPfR9awYq2',
      description: 'HBtoUA5sTkPZRo5dkkP01WgFX4A6yPflFRtG3nZOAaWZ7Pipe3xIgvBRdLTY',
    },
  );

  const transaction = new Transaction();
  transaction.add(instruction);
  transaction.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  transaction.feePayer = payer.publicKey;
  transaction.partialSign(store);

  const createStoreRes = await transactionHandler.sendAndConfirmTransaction(
    transaction,
    [store],
    defaultSendOptions,
  );
  addLabel('create:store', store);

  assertConfirmedTransaction(t, createStoreRes.txConfirmed);
});

test('create-store: bad name', async (t) => {
  const payer = Keypair.generate();
  const store = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  await airdrop(connection, payer.publicKey, 2);

  t.throws(() =>
    createCreateStoreInstruction(
      {
        store: store.publicKey,
        admin: payer.publicKey,
      },
      {
        name: 'izd5Pr9ltIAJL4ac8cYMUDlakSXNPnJPfR9awYq2',
        description: '',
      },
    ),
  );
});

test('create-store: bad description', async (t) => {
  const payer = Keypair.generate();
  const store = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  await airdrop(connection, payer.publicKey, 2);

  t.throws(() =>
    createCreateStoreInstruction(
      {
        store: store.publicKey,
        admin: payer.publicKey,
      },
      {
        name: 'izd5Pr9ltIAJL4ac8cYMUDlakSXNPnJPfR9awYq2',
        description: '',
      },
    ),
  );
});
