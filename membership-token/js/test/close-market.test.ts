import test from 'tape';
import { killStuckProcess, sleep } from './utils';

import {
  mintNFT,
  createStore,
  createPrerequisites,
  initSellingResource,
  createMarket,
  closeMarket,
  closeMarketLimitedDuration,
} from './actions';

killStuckProcess();

test('close-market: success', async (t) => {
  const { payer, connection, transactionHandler } = await createPrerequisites();

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

  const { sellingResource } = await initSellingResource({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    maxSupply: 100,
  });

  const { mint: treasuryMint } = await mintNFT({
    transactionHandler,
    payer,
    connection,
  });

  const startDate = Math.round(Date.now() / 1000) + 2;
  const params = {
    name: 'Market',
    description: '',
    startDate,
    endDate: null,
    mutable: true,
    price: 0.001,
    piecesInOneWallet: 1,
  };

  const { market } = await createMarket({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    sellingResource: sellingResource.publicKey,
    treasuryMint: treasuryMint.publicKey,
    params,
  });

  await sleep(3000);

  console.log('Payer: ', payer.publicKey.toBase58());
  await closeMarket({
    test: t,
    transactionHandler,
    payer,
    connection,
    market,
  });
});

test('close-market: fail, market has the specific endDate', async (t) => {
  const { payer, connection, transactionHandler } = await createPrerequisites();

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

  const { sellingResource } = await initSellingResource({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    maxSupply: 100,
  });

  const { mint: treasuryMint } = await mintNFT({
    transactionHandler,
    payer,
    connection,
  });

  const startDate = Math.round(Date.now() / 1000) + 2;
  const params = {
    name: 'Market',
    description: '',
    startDate,
    endDate: startDate + 4000,
    mutable: true,
    price: 0.001,
    piecesInOneWallet: 1,
  };

  const { market } = await createMarket({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    sellingResource: sellingResource.publicKey,
    treasuryMint: treasuryMint.publicKey,
    params,
  });

  await sleep(3000);

  await closeMarketLimitedDuration({
    test: t,
    transactionHandler,
    payer,
    connection,
    market,
  });
});
