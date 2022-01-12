import test from 'tape';
import { Connection, Keypair } from '@solana/web3.js';
import { connectionURL, killStuckProcess } from './utils';
import {
  airdrop,
  assertConfirmedTransaction,
  PayerTransactionHandler,
  defaultSendOptions,
} from '@metaplex-foundation/amman';

import { addLabel } from './utils/address-labels';

killStuckProcess();

// TODO: at this point only success cases are tested, however tests for
// incorrect inputs, etc. should be added ASAP
test('create-metadata-account: success', async (t) => {
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

  const createStoreRes = await transactionHandler.sendAndConfirmTransaction(
    instruction,
    [store],
    defaultSendOptions,
  );
  addLabel('create:store', store);

  assertConfirmedTransaction(t, createStoreRes.txConfirmed);
});
