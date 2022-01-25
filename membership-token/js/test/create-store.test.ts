import test from 'tape';
import { Connection, Keypair } from '@solana/web3.js';
import { connectionURL, killStuckProcess, logDebug } from './utils';
import {
  airdrop,
  assertConfirmedTransaction,
  PayerTransactionHandler,
  defaultSendOptions,
} from '@metaplex-foundation/amman';

import { addLabel } from './utils';
import { createCreateStoreInstruction } from '../src/mpl-membership-token';
import { DESCRIPTION_MAX_LEN, NAME_MAX_LEN } from '../src/consts';

import { createStoreTransaction } from './transactions/create-store';

killStuckProcess();

test('create-store: success', async (t) => {
  const payer = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const { store, transaction } = await createStoreTransaction(payer, connection);

  const createStoreRes = await transactionHandler.sendAndConfirmTransaction(
    transaction,
    [store],
    defaultSendOptions,
  );
  addLabel('create:store', store);
  logDebug(`store: ${store.publicKey}`);
  logDebug(createStoreRes.txSummary.logMessages.join('\n'));

  assertConfirmedTransaction(t, createStoreRes.txConfirmed);
});

test('create-store: name length is longer the specification value', async (t) => {
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
        name: 'n'.repeat(NAME_MAX_LEN + 1),
        description: 'd'.repeat(DESCRIPTION_MAX_LEN - 3),
      },
    ),
  );
});

test('create-store: description length is longer the specification value', async (t) => {
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
        name: 'n'.repeat(NAME_MAX_LEN - 2),
        description: 'd'.repeat(DESCRIPTION_MAX_LEN + 1),
      },
    ),
  );
});
