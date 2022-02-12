import test from 'tape';

import { killStuckProcess } from './utils';
import { createStore, createPrerequisites } from './actions';

killStuckProcess();

test('create-store: success', async (t) => {
  const { payer, connection, transactionHandler } = await createPrerequisites();
  console.log("🚀 ~ file: createStore.test.ts ~ line 10 ~ test ~ transactionHandler", transactionHandler)
  console.log("🚀 ~ file: createStore.test.ts ~ line 10 ~ test ~ connection", connection["_rpcEndpoint"])
  console.log("🚀 ~ file: createStore.test.ts ~ line 10 ~ test ~ payer", payer.publicKey.toBase58())

  const store = await createStore({
    test: t,
    transactionHandler,
    payer,
    connection,
    params: {
      name: 'Store',
      description: 'Description',
    },
  });
  console.log("🚀 ~ file: createStore.test.ts ~ line 24 ~ test ~ store", store.publicKey.toBase58())
});

test('create-store: short name and empty description', async (t) => {
  const { payer, connection, transactionHandler } = await createPrerequisites();

  await createStore({
    test: t,
    transactionHandler,
    payer,
    connection,
    params: {
      name: 'Store',
      description: '',
    },
  });
});
