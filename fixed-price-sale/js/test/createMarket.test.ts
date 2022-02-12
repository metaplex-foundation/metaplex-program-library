import test from 'tape';
import { killStuckProcess } from './utils';

import {
  mintNFT,
  createStore,
  createPrerequisites,
  initSellingResource,
  createMarket,
} from './actions';

killStuckProcess();

test('create-market: success', async (t) => {
  const { payer, connection, transactionHandler } = await createPrerequisites();
  console.log("🚀 ~ file: createMarket.test.ts ~ line 16 ~ test ~ transactionHandler", transactionHandler)
  console.log("🚀 ~ file: createMarket.test.ts ~ line 16 ~ test ~ connection", connection["_rpcEndpoint"])
  console.log("🚀 ~ file: createMarket.test.ts ~ line 16 ~ test ~ payer", payer.publicKey.toBase58())

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
  console.log("🚀 ~ file: createMarket.test.ts ~ line 30 ~ test ~ store", store.publicKey.toBase58())

  const { sellingResource } = await initSellingResource({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    maxSupply: 100,
  });
  console.log("🚀 ~ file: createMarket.test.ts ~ line 33 ~ test ~ sellingResource", sellingResource.publicKey.toBase58())


  const { mint: treasuryMint } = await mintNFT({
    transactionHandler,
    payer,
    connection,
  });
  console.log("🚀 ~ file: createMarket.test.ts ~ line 43 ~ test ~ treasuryMint", treasuryMint.publicKey.toBase58())


  const startDate = Math.round(Date.now() / 1000) + 5;
  const params = {
    name: 'Market',
    description: '',
    startDate,
    endDate: startDate + 5 * 20,
    mutable: true,
    price: 0.001,
    piecesInOneWallet: 1,
  };

  const { market, treasuryHolder } = await createMarket({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    sellingResource: sellingResource.publicKey,
    treasuryMint: treasuryMint.publicKey,
    params,
  });
  console.log("🚀 ~ file: createMarket.test.ts ~ line 63 ~ test ~ treasuryHolder", treasuryHolder.publicKey.toBase58())
  console.log("🚀 ~ file: createMarket.test.ts ~ line 63 ~ test ~ market", market.publicKey.toBase58())
  
});
