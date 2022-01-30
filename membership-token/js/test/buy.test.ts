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
import { mintTokenToAccount } from './actions/mint-token-to-account';
import { connectionURL, killStuckProcess, logDebug, sleep } from './utils';

killStuckProcess();

test('buy: successful purchase for newly minted treasury mint', async (t) => {
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

  logDebug('buy:: minted resource mint', resourceMint.publicKey.toBase58());

  const resourceMintMetadata = await Metadata.getPDA(resourceMint.publicKey);

  const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));

  const [vaultOwner, vaultOwnerBump] = await findVaultOwnerAddress(
    resourceMint.publicKey,
    store.publicKey,
  );

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
    [sellingResource],
    defaultSendOptions,
  );

  logDebug('buy:: created selling resource', sellingResource.publicKey.toBase58());
  assertConfirmedTransaction(t, initSellingResourceRes.txConfirmed);

  const { mint: treasuryMint, tokenAccount: userTokenAcc } = await mintNFT({
    transactionHandler,
    payer: payer.publicKey,
    connection,
  });

  logDebug('buy:: minted treasury mint', treasuryMint.publicKey.toBase58());

  const [treasuryOwner, treasuryOwnerBump] = await findTresuryOwnerAddress(
    treasuryMint.publicKey,
    sellingResource.publicKey,
  );

  const { tokenAccount: treasuryHolder, createTokenTx: createTreasuryTx } =
    await createTokenAccount({
      payer: payer.publicKey,
      connection,
      mint: treasuryMint.publicKey,
      owner: treasuryOwner,
    });

  await transactionHandler.sendAndConfirmTransaction(
    createTreasuryTx,
    [treasuryHolder],
    defaultSendOptions,
  );

  logDebug('buy:: treasuryHolder', treasuryHolder.publicKey.toBase58());

  const startDate = Math.round(Date.now() / 1000);

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
    price: 0.1,
    piecesInOneWallet: 10,
    startDate: startDate,
    endDate: startDate + 100000,
  });

  const marketRes = await transactionHandler.sendAndConfirmTransaction(
    marketTx,
    [market, payer],
    defaultSendOptions,
  );

  logDebug('buy:: created market', market.publicKey.toBase58());
  assertConfirmedTransaction(t, marketRes.txConfirmed);

  const [tradeHistory, tradeHistoryBump] = await findTradeHistoryAddress(
    payer.publicKey,
    market.publicKey,
  );

  const { mint: newMint } = await mintTokenToAccount({
    connection,
    payer: payer.publicKey,
    transactionHandler,
  });

  logDebug('buy:: new mint', newMint.publicKey.toBase58());

  const newMintEdition = await Edition.getPDA(newMint.publicKey);
  const newMintMetadata = await Metadata.getPDA(newMint.publicKey);

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
