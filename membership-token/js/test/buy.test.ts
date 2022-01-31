import BN from 'bn.js';
import test from 'tape';
import { assertConfirmedTransaction, defaultSendOptions } from '@metaplex-foundation/amman';
import { Edition, EditionMarker, Metadata } from '@metaplex-foundation/mpl-token-metadata';

import { findTradeHistoryAddress } from '../src/utils';
import { createBuyTransaction } from './transactions';
import { mintNFT } from './actions/mint-nft';
import { mintTokenToAccount } from './actions/mint-token-to-account';
import { killStuckProcess, logDebug, sleep } from './utils';
import { createPrerequisites, createStore, initSellingResource } from './actions';
import { createMarket } from './actions/create-market';

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

  const { sellingResource, vault, vaultOwner, vaultOwnerBump, resourceMint } =
    await initSellingResource({
      test: t,
      transactionHandler,
      payer,
      connection,
      store: store.publicKey,
      maxSupply: 100,
    });

  const { mint: treasuryMint, tokenAccount: userTokenAcc } = await mintNFT({
    transactionHandler,
    payer,
    connection,
  });

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

  const [tradeHistory, tradeHistoryBump] = await findTradeHistoryAddress(
    payer.publicKey,
    market.publicKey,
  );

  const { mint: newMint } = await mintTokenToAccount({
    connection,
    payer: payer.publicKey,
    transactionHandler,
  });

  logDebug('new mint', newMint.publicKey.toBase58());

  const newMintEdition = await Edition.getPDA(newMint.publicKey);
  const newMintMetadata = await Metadata.getPDA(newMint.publicKey);

  const resourceMintMasterEdition = await Edition.getPDA(resourceMint.publicKey);
  const resourceMintMetadata = await Metadata.getPDA(resourceMint.publicKey);
  const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));

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

  const buyRes = await transactionHandler.sendAndConfirmTransaction(
    buyTx,
    [payer],
    defaultSendOptions,
  );

  logDebug('buy:: successful purchase');
  assertConfirmedTransaction(t, buyRes.txConfirmed);
});
