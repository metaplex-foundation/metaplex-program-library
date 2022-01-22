import BN from 'bn.js';
import test from 'tape';
import {
  airdrop,
  assertConfirmedTransaction,
  defaultSendOptions,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman';
import { Connection, Keypair } from '@solana/web3.js';
import { Edition, EditionMarker, Metadata } from '@metaplex-foundation/mpl-token-metadata';

import {
  findVaultOwnerAddress,
  findTradeHistoryAddress,
  findTresuryOwnerAddress,
} from '../src/utils';
import {
  createTokenAccount,
  createBuyTransaction,
  createStoreTransaction,
  createMarketTransaction,
  createInitSellingResourceTransaction,
} from './transactions';
import { mintNFT } from './actions/mint-nft';
import { addLabel, connectionURL, logDebug } from './utils';

test('buy: success', async (t) => {
  const payer = Keypair.generate();

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const { store, transaction: createStoreTx } = await createStoreTransaction(payer, connection);

  const createStoreRes = await transactionHandler.sendAndConfirmTransaction(
    createStoreTx,
    [store],
    defaultSendOptions,
  );

  logDebug('buy:: created store', store.publicKey.toBase58());
  addLabel('create:store', store.publicKey.toBase58());
  assertConfirmedTransaction(t, createStoreRes.txConfirmed);

  const {
    edition: masterEdition,
    editionBump: masterEditionBump,
    tokenAccount: resourceToken,
    mint: resourceMint,
  } = await mintNFT({
    transactionHandler,
    payer: payer.publicKey,
    connection,
  });

  const resourceMintMetadata = await Metadata.getPDA(resourceMint.publicKey);

  addLabel('get:resource-mint-metadata', resourceMintMetadata);

  const resourceMintMasterMetadata = await Metadata.getPDA(masterEdition);

  addLabel('get:resource-mint-master-edition-metadata', resourceMintMasterMetadata);

  const resourceMintEdition = await Edition.getPDA(resourceMint.publicKey);

  addLabel('get:resource-mint-edition', resourceMintEdition);

  const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));

  addLabel('get:resource-mint-edition-marker', resourceMintEditionMarker);

  const [vaultOwner, vaultOwnerBump] = await findVaultOwnerAddress(
    resourceMint.publicKey,
    store.publicKey,
  );

  addLabel('get:vault-owner', vaultOwner);

  const { tokenAccount: vault, createTokenTx: createVaultTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: resourceMint.publicKey,
    connection,
    owner: vaultOwner,
  });

  const createVaultRes = await transactionHandler.sendAndConfirmTransaction(
    createVaultTx,
    [vault],
    defaultSendOptions,
  );

  logDebug('buy:: created vault', vault.publicKey.toBase58());
  addLabel('create:vault', vault.publicKey.toBase58());
  assertConfirmedTransaction(t, createVaultRes.txConfirmed);

  const { initSellingResourceTx, sellingResource } = await createInitSellingResourceTransaction({
    payer,
    connection,
    store,
    masterEdition,
    masterEditionBump,
    resourceMint: resourceMint.publicKey,
    resourceToken: resourceToken.publicKey,
    vault,
    owner: vaultOwner,
    vaultOwnerBump,
    maxSupply: 100,
  });

  const initSellingResourceRes = await transactionHandler.sendAndConfirmTransaction(
    initSellingResourceTx,
    [sellingResource, vault],
    defaultSendOptions,
  );

  logDebug('buy:: created selling resource', sellingResource.publicKey.toBase58());
  addLabel('create:selling-resource', sellingResource.publicKey.toBase58());
  assertConfirmedTransaction(t, initSellingResourceRes.txConfirmed);

  const { mint: treasuryMint } = await mintNFT({
    transactionHandler,
    payer: payer.publicKey,
    connection,
  });

  addLabel('create:market-treasury-mint', treasuryMint);

  const [treasuryOwner, treasuryOwnerBump] = await findTresuryOwnerAddress(
    treasuryMint.publicKey,
    sellingResource.publicKey,
  );

  addLabel('get:market-treasury-owner', treasuryOwner);

  const { tokenAccount: treasuryHolder, createTokenTx: createTreasuryTx } =
    await createTokenAccount({
      payer: payer.publicKey,
      connection,
      mint: treasuryMint.publicKey,
      owner: treasuryOwner,
    });

  const createTreasuryRes = await transactionHandler.sendAndConfirmTransaction(
    createTreasuryTx,
    [treasuryHolder],
    defaultSendOptions,
  );

  logDebug('buy:: created market treasury holder', treasuryHolder.publicKey.toBase58());
  addLabel('create:market-treasury-holder', treasuryHolder.publicKey.toBase58());
  assertConfirmedTransaction(t, createTreasuryRes.txConfirmed);

  const { marketTx, market } = await createMarketTransaction({
    payer,
    connection,
    store,
    sellingResourceOwner: payer,
    sellingResource: sellingResource.publicKey,
    mint: treasuryMint.publicKey,
    treasuryHolder: treasuryHolder.publicKey,
    owner: treasuryOwner,
    treasuryOwnerBump,
    name: 'Market Name',
    description: 'Market Description',
    mutable: true,
    price: 1,
    piecesInOneWallet: 10,
    startDate: Math.round(Date.now() / 1000) + 5,
    endDate: Math.round(Date.now() / 1000) + 10 * 20,
  });

  const marketRes = await transactionHandler.sendAndConfirmTransaction(
    marketTx,
    [market, payer],
    defaultSendOptions,
  );

  logDebug('buy:: created market', market.publicKey.toBase58());
  addLabel('create:market', market.publicKey.toBase58());
  assertConfirmedTransaction(t, marketRes.txConfirmed);

  const [tradeHistory, tradeHistoryBump] = await findTradeHistoryAddress(
    payer.publicKey,
    market.publicKey,
  );

  addLabel('get:trade-history', tradeHistory);

  const { tx: buyTx } = await createBuyTransaction({
    buyer: payer.publicKey,
    buyerTokenAccount: store.publicKey,
    connection,
    sellingResource: sellingResource.publicKey,
    market: market.publicKey,
    marketTreasuryHolder: treasuryHolder.publicKey,
    tradeHistory,
    tradeHistoryBump,
    vault: vault.publicKey,
    vaultOwner,
    vaultOwnerBump,
    resourceMint: resourceMint.publicKey,
    resourceMintEdition,
    resourceMintEditionMarker,
    resourceMintMetadata,
    resourceMintMasterEdition: masterEdition,
    resourceMintMasterMetadata,
  });

  const buyRes = await transactionHandler.sendAndConfirmTransaction(
    buyTx,
    [payer, vault],
    defaultSendOptions,
  );

  logDebug('buy:: successful purchase');
  assertConfirmedTransaction(t, buyRes.txConfirmed);
});
