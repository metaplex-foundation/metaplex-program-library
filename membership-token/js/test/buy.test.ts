import BN from 'bn.js';
import test from 'tape';
import {
  airdrop,
  assertConfirmedTransaction,
  defaultSendOptions,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
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
import { mintTokenToAccount } from './actions/mint-token-to-account';

test('buy: success purchase with native SOL', async (t) => {
  const payer = Keypair.generate();

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 5);

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
    edition: resourceMintMasterEdition,
    editionBump: resourceMasterEditionBump,
    tokenAccount: resourceToken,
    mint: resourceMint,
  } = await mintNFT({
    transactionHandler,
    payer: payer.publicKey,
    connection,
  });

  const resourceMintMasterMetadata = await Metadata.getPDA(resourceMintMasterEdition);

  const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));

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
    masterEdition: resourceMintMasterEdition,
    masterEditionBump: resourceMasterEditionBump,
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

  const treasuryMint = new PublicKey('11111111111111111111111111111111');

  const [treasuryOwner, treasuryOwnerBump] = await findTresuryOwnerAddress(
    treasuryMint,
    sellingResource.publicKey,
  );

  addLabel('get:market-treasury-owner', treasuryOwner);

  const { marketTx, market } = await createMarketTransaction({
    payer,
    connection,
    store,
    sellingResourceOwner: payer,
    sellingResource: sellingResource.publicKey,
    mint: treasuryMint,
    treasuryHolder: treasuryOwner,
    owner: treasuryOwner,
    treasuryOwnerBump,
    name: 'Market Name',
    description: 'Market Description',
    mutable: true,
    price: 1,
    piecesInOneWallet: 10,
    startDate: Math.round(Date.now() / 1000),
    endDate: Math.round(Date.now() / 1000) + 5 * 20,
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

  const { mint: newMint } = await mintTokenToAccount({
    connection,
    payer: payer.publicKey,
    transactionHandler,
  });

  const newMintEdition = await Edition.getPDA(newMint.publicKey);
  const newMintMetadata = await Metadata.getPDA(newMint.publicKey);

  const { tx: buyTx } = await createBuyTransaction({
    connection,
    buyer: payer.publicKey,
    userTokenAccount: payer.publicKey,
    resourceMintEditionMarker,
    resourceMintMasterEdition,
    resourceMintMasterMetadata,
    sellingResource: sellingResource.publicKey,
    market: market.publicKey,
    marketTreasuryHolder: treasuryOwner,
    treasuryOwner,
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
