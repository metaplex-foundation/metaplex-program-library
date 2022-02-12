import BN from 'bn.js';
import test from 'tape';
import { assertConfirmedTransaction, defaultSendOptions } from '@metaplex-foundation/amman';
import { Edition, EditionMarker, Metadata } from '@metaplex-foundation/mpl-token-metadata';

import { findTradeHistoryAddress } from '../src/utils';
import { createBuyTransaction } from './transactions';
import {
  createPrerequisites,
  createStore,
  initSellingResource,
  createMarket,
  mintNFT,
  mintTokenToAccount,
} from './actions';
import { killStuckProcess, logDebug, sleep } from './utils';

killStuckProcess();

test('buy: successful purchase for newly minted treasury mint', async (t) => {
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
  console.log("🚀 ~ file: buy.test.ts ~ line 33 ~ test ~ store", store.publicKey.toBase58())

  const { sellingResource, vault, vaultOwner, vaultOwnerBump, resourceMint } =
  
    await initSellingResource({
      test: t,
      transactionHandler,
      payer,
      connection,
      store: store.publicKey,
      maxSupply: 100,
    });

  console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ resourceMint", resourceMint.publicKey.toBase58())
  console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ vaultOwnerBump", vaultOwnerBump)
  console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ vaultOwner", vaultOwner.toBase58())
  console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ vault", vault.publicKey.toBase58())
  console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ sellingResource", sellingResource.publicKey.toBase58())

  const { mint: treasuryMint, tokenAccount: userTokenAcc } = await mintNFT({
    transactionHandler,
    payer,
    connection,
  });
  console.log("🚀 ~ file: buy.test.ts ~ line 51 ~ test ~ treasuryMint", treasuryMint.publicKey.toBase58())
  console.log("🚀 ~ file: buy.test.ts ~ line 56 ~ test ~ userTokenAcc", userTokenAcc.publicKey.toBase58())

  const startDate = Math.round(Date.now() / 1000);
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
  console.log("🚀 ~ file: buy.test.ts ~ line 72 ~ test ~ treasuryHolder", treasuryHolder.publicKey.toBase58())
  console.log("🚀 ~ file: buy.test.ts ~ line 72 ~ test ~ market", market.publicKey.toBase58())

  const [tradeHistory, tradeHistoryBump] = await findTradeHistoryAddress(
    payer.publicKey,
    market.publicKey,
  );
  console.log("🚀 ~ file: buy.test.ts ~ line 85 ~ test ~ tradeHistoryBump", tradeHistoryBump)
  console.log("🚀 ~ file: buy.test.ts ~ line 88 ~ test ~ tradeHistory", tradeHistory.toBase58())

  const { mint: newMint } = await mintTokenToAccount({
    connection,
    payer: payer.publicKey,
    transactionHandler,
  });
  console.log("🚀 ~ file: buy.test.ts ~ line 92 ~ test ~ newMint", newMint.publicKey.toBase58())


  logDebug('new mint', newMint.publicKey.toBase58());

  const newMintEdition = await Edition.getPDA(newMint.publicKey);
  console.log("🚀 ~ file: buy.test.ts ~ line 101 ~ test ~ newMintEdition", newMintEdition.toBase58())
  const newMintMetadata = await Metadata.getPDA(newMint.publicKey);
  console.log("🚀 ~ file: buy.test.ts ~ line 103 ~ test ~ newMintMetadata", newMintMetadata.toBase58())

  const resourceMintMasterEdition = await Edition.getPDA(resourceMint.publicKey);
  console.log("🚀 ~ file: buy.test.ts ~ line 106 ~ test ~ resourceMintMasterEdition", resourceMintMasterEdition.toBase58())
  const resourceMintMetadata = await Metadata.getPDA(resourceMint.publicKey);
  console.log("🚀 ~ file: buy.test.ts ~ line 108 ~ test ~ resourceMintMetadata", resourceMintMetadata.toBase58())
  const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));
  console.log("🚀 ~ file: buy.test.ts ~ line 109 ~ test ~ resourceMintEditionMarker", resourceMintEditionMarker.toBase58())

  await sleep(1000);

  const { tx: buyTx } = await createBuyTransaction({
    connection,
    buyer: payer.publicKey,
    userTokenAccount: userTokenAcc.publicKey,
    resourceMintMetadata,
    resourceMintEditionMarker,
    resourceMintMasterEdition,
    sellingResource: sellingResource.publicKey,
    market: market.publicKey,
    marketTreasuryHolder: treasuryHolder.publicKey,
    vaultOwner,
    tradeHistory,
    tradeHistoryBump,
    vault: vault.publicKey,
    vaultOwnerBump,
    newMint: newMint.publicKey,
    newMintEdition,
    newMintMetadata,
  });
  console.log("🚀 ~ file: buy.test.ts ~ line 116 ~ test ~ buyTx", buyTx)


  const buyRes = await transactionHandler.sendAndConfirmTransaction(
    buyTx,
    [payer],
    defaultSendOptions,
  );
  console.log("🚀 ~ file: buy.test.ts ~ line 142 ~ test ~ buyRes", buyRes.txSignature)

  console.log("End to End complete for buy method")

  logDebug('buy:: successful purchase');
  assertConfirmedTransaction(t, buyRes.txConfirmed);
});
