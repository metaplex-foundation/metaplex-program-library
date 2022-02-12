import test from 'tape';

import { killStuckProcess } from './utils';
import { createPrerequisites, createStore, initSellingResource } from './actions';

killStuckProcess();

test('init-selling-resource: success', async (t) => {
  const { payer, connection, transactionHandler } = await createPrerequisites();
  console.log("🚀 ~ file: initSellingResource.test.ts ~ line 10 ~ test ~ transactionHandler", transactionHandler)
  console.log("🚀 ~ file: initSellingResource.test.ts ~ line 10 ~ test ~ connection", connection["_rpcEndpoint"])
  console.log("🚀 ~ file: initSellingResource.test.ts ~ line 10 ~ test ~ payer", payer.publicKey.toBase58())

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
  console.log("🚀 ~ file: initSellingResource.test.ts ~ line 24 ~ test ~ store", store.publicKey.toBase58())

  const {
    sellingResource,
    vault,
    vaultOwner,
    vaultOwnerBump,
    resourceMint,
    metadata,
  } = await initSellingResource({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    maxSupply: 100,
  });
    console.log("🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ metadata", metadata.toBase58())
    console.log("🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ resourceMint", resourceMint.publicKey.toBase58())
    console.log("🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ vaultOwnerBump", vaultOwnerBump)
    console.log("🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ vaultOwner", vaultOwner.toBase58())
    console.log("🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ vault", vault.publicKey.toBase58())
    console.log("🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ sellingResource", sellingResource.publicKey.toBase58())
  
});
