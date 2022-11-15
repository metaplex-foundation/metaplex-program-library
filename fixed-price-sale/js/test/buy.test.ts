import test from 'tape';

import { findTradeHistoryAddress } from '../src/utils';
import { createBuyTransaction } from './transactions';
import { killStuckProcess, logDebug, sleep } from './utils';
import {
  createPrerequisites,
  createStore,
  initSellingResource,
  mintNFT,
  createMarket,
  mintTokenToAccount,
} from './actions';
import { CreateMarketInstructionArgs, GatingTokenMissingError } from '../src';
import { verifyCollection } from './actions/verifyCollection';
import { Metaplex, toBigNumber } from '@metaplex-foundation/js';

killStuckProcess();

test('buy: successful purchase without gating', async (t) => {
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
  const params: Omit<CreateMarketInstructionArgs, 'treasuryOwnerBump'> = {
    name: 'Market',
    description: '',
    startDate,
    endDate: startDate + 5 * 20,
    mutable: true,
    price: 0.001,
    piecesInOneWallet: 1,
    // No gating
    gatingConfig: null,
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

  const { mint: newMint, mintAta } = await mintTokenToAccount({
    connection,
    payer: payer.publicKey,
    transactionHandler,
  });

  logDebug('new mint', newMint.publicKey.toBase58());

  const metaplex = Metaplex.make(connection);
  const pdas = metaplex.nfts().pdas();
  const newMintEdition = pdas.edition({ mint: newMint.publicKey });
  const newMintMetadata = pdas.metadata({ mint: newMint.publicKey });

  const resourceMintMasterEdition = pdas.edition({ mint: resourceMint.publicKey });
  const resourceMintMetadata = pdas.metadata({ mint: resourceMint.publicKey });
  const resourceMintEditionMarker = pdas.editionMarker({
    mint: resourceMint.publicKey,
    edition: toBigNumber(1),
  });

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
    newTokenAccount: mintAta.publicKey,
  });

  await transactionHandler.sendAndConfirmTransaction(buyTx, [payer]).assertSuccess(t);
  logDebug('buy:: successful purchase');
});

test('buy: successful purchase with gating', async (t) => {
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

  // Create collection
  const {
    mint: collectionMint,
    metadata: collectionMetadata,
    edition: collectionMasterEditionAccount,
  } = await mintNFT({
    transactionHandler,
    payer,
    connection,
    maxSupply: 0,
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
  const params: Omit<CreateMarketInstructionArgs, 'treasuryOwnerBump'> = {
    name: 'Market',
    description: '',
    startDate,
    endDate: startDate + 5 * 20,
    mutable: true,
    price: 0.001,
    piecesInOneWallet: 1,
    // Assign gating to market to use collection
    gatingConfig: {
      collection: collectionMint.publicKey,
      expireOnUse: true,
      gatingTime: null,
    },
  };

  const { market, treasuryHolder } = await createMarket({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    sellingResource: sellingResource.publicKey,
    treasuryMint: treasuryMint.publicKey,
    collectionMint: collectionMint.publicKey,
    params,
  });

  const [tradeHistory, tradeHistoryBump] = await findTradeHistoryAddress(
    payer.publicKey,
    market.publicKey,
  );

  const { mint: newMint, mintAta } = await mintTokenToAccount({
    connection,
    payer: payer.publicKey,
    transactionHandler,
  });

  logDebug('new mint', newMint.publicKey.toBase58());

  const metaplex = Metaplex.make(connection);
  const pdas = metaplex.nfts().pdas();
  const newMintEdition = pdas.edition({ mint: newMint.publicKey });
  const newMintMetadata = pdas.metadata({ mint: newMint.publicKey });

  const resourceMintMasterEdition = pdas.edition({ mint: resourceMint.publicKey });
  const resourceMintMetadata = pdas.metadata({ mint: resourceMint.publicKey });
  const resourceMintEditionMarker = pdas.editionMarker({
    mint: resourceMint.publicKey,
    edition: toBigNumber(1),
  });

  // Create NFT from collection
  const {
    mint: userCollectionTokenMint,
    tokenAccount: userCollectionTokenAcc,
    metadata: userCollectionMetadata,
  } = await mintNFT({
    transactionHandler,
    payer,
    connection,
    collectionMint: collectionMint.publicKey,
  });

  await verifyCollection({
    transactionHandler,
    connection,
    payer,
    metadata: userCollectionMetadata,
    collectionAuthority: payer.publicKey,
    collection: collectionMetadata,
    collectionMint: collectionMint.publicKey,
    collectionMasterEditionAccount,
  });

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
    newTokenAccount: mintAta.publicKey,
    additionalKeys: [
      {
        pubkey: userCollectionTokenAcc.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: userCollectionTokenMint.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: userCollectionMetadata,
        isSigner: false,
        isWritable: false,
      },
    ],
  });

  await transactionHandler.sendAndConfirmTransaction(buyTx, [payer]).assertSuccess(t);
  logDebug('buy:: successful purchase');
});

test('buy: unsuccessful purchase with gating', async (t) => {
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

  // Create collection
  const { mint: collectionMint } = await mintNFT({
    transactionHandler,
    payer,
    connection,
    maxSupply: 0,
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
  const params: Omit<CreateMarketInstructionArgs, 'treasuryOwnerBump'> = {
    name: 'Market',
    description: '',
    startDate,
    endDate: startDate + 5 * 20,
    mutable: true,
    price: 0.001,
    piecesInOneWallet: 1,
    // Assign gating to market to use collection
    gatingConfig: {
      collection: collectionMint.publicKey,
      expireOnUse: true,
      gatingTime: null,
    },
  };

  const { market, treasuryHolder } = await createMarket({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    sellingResource: sellingResource.publicKey,
    treasuryMint: treasuryMint.publicKey,
    collectionMint: collectionMint.publicKey,
    params,
  });

  const [tradeHistory, tradeHistoryBump] = await findTradeHistoryAddress(
    payer.publicKey,
    market.publicKey,
  );

  const { mint: newMint, mintAta } = await mintTokenToAccount({
    connection,
    payer: payer.publicKey,
    transactionHandler,
  });

  logDebug('new mint', newMint.publicKey.toBase58());

  const metaplex = Metaplex.make(connection);
  const pdas = metaplex.nfts().pdas();
  const newMintEdition = pdas.edition({ mint: newMint.publicKey });
  const newMintMetadata = pdas.metadata({ mint: newMint.publicKey });

  const resourceMintMasterEdition = pdas.edition({ mint: resourceMint.publicKey });
  const resourceMintMetadata = pdas.metadata({ mint: resourceMint.publicKey });
  const resourceMintEditionMarker = pdas.editionMarker({
    mint: resourceMint.publicKey,
    edition: toBigNumber(1),
  });

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
    newTokenAccount: mintAta.publicKey,
    // User doesn't have gating token
  });

  await transactionHandler
    .sendAndConfirmTransaction(buyTx, [payer])
    .assertError(t, GatingTokenMissingError);
});
