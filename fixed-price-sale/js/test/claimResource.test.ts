import BN from 'bn.js';
import test from 'tape';
import { getAccount, getAssociatedTokenAddress } from '@solana/spl-token';
import {
  assertConfirmedTransaction,
  assertError,
  defaultSendOptions,
} from '@metaplex-foundation/amman';
import { Edition, EditionMarker, Metadata } from '@metaplex-foundation/mpl-token-metadata';
import { findPayoutTicketAddress, findTradeHistoryAddress } from '../src/utils';
import {
  createPrerequisites,
  createStore,
  initSellingResource,
  createMarket,
  mintNFT,
  mintTokenToAccount,
} from './actions';

import {
  closeMarket,
  createBuyTransaction,
  createTokenAccount,
  createClaimResourceTransaction,
  createWithdrawTransaction,
} from './transactions';
import { killStuckProcess, logDebug, sleep } from './utils';

killStuckProcess();

test('claim resource: success', async (t) => {
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

  const {
    sellingResource,
    vault,
    vaultOwner,
    vaultOwnerBump,
    resourceMint,
    primaryMetadataCreators,
  } = await initSellingResource({
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

  const startDate = Math.round(Date.now() / 1000) + 1;
  const params = {
    name: 'Market',
    description: '',
    startDate,
    endDate: null,
    mutable: true,
    price: 1,
    piecesInOneWallet: 1,
  };

  const { market, treasuryHolder, treasuryOwnerBump, treasuryOwner } = await createMarket({
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

  const newMintEdition = await Edition.getPDA(newMint.publicKey);
  const newMintMetadata = await Metadata.getPDA(newMint.publicKey);

  const resourceMintMasterEdition = await Edition.getPDA(resourceMint.publicKey);
  const resourceMintMetadata = await Metadata.getPDA(resourceMint.publicKey);
  const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));

  await sleep(1000);

  // TODO: this transaction fails with: `MarketIsNotStarted. Error Number: 6013. Error Message: Market is not started.`
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

  const buyRes = await transactionHandler.sendAndConfirmTransaction(
    buyTx,
    [payer],
    defaultSendOptions,
  );

  logDebug('buy:: successful purchase');
  assertConfirmedTransaction(t, buyRes.txConfirmed);

  await sleep(3000);

  const marketTx = await closeMarket({
    transactionHandler,
    payer,
    connection,
    market,
  });

  const marketRes = await transactionHandler.sendAndConfirmTransaction(
    marketTx,
    [payer],
    defaultSendOptions,
  );

  logDebug(`market: ${market.publicKey}`);
  assertConfirmedTransaction(t, marketRes.txConfirmed);

  const [payoutTicket, payoutTicketBump] = await findPayoutTicketAddress(
    market.publicKey,
    payer.publicKey,
  );

  const destination = await getAssociatedTokenAddress(treasuryMint.publicKey, payer.publicKey);

  const metadata = await Metadata.getPDA(resourceMint.publicKey);

  const withdrawTx = await createWithdrawTransaction({
    connection,
    payer,
    market: market.publicKey,
    sellingResource: sellingResource.publicKey,
    metadata,
    treasuryHolder: treasuryHolder.publicKey,
    treasuryMint: treasuryMint.publicKey,
    destination,
    payoutTicket,
    payoutTicketBump,
    treasuryOwnerBump,
    treasuryOwner,
    primaryMetadataCreators,
  });

  const withdrawRes = await transactionHandler.sendAndConfirmTransaction(
    withdrawTx,
    [payer],
    defaultSendOptions,
  );

  assertConfirmedTransaction(t, withdrawRes.txConfirmed);

  const { tokenAccount: claimToken, createTokenTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: resourceMint.publicKey,
    connection,
  });

  const claimTokenRes = await transactionHandler.sendAndConfirmTransaction(
    createTokenTx,
    [claimToken],
    defaultSendOptions,
  );

  assertConfirmedTransaction(t, claimTokenRes.txConfirmed);

  const claimResourceTx = await createClaimResourceTransaction({
    connection,
    payer,
    market: market.publicKey,
    sellingResource: sellingResource.publicKey,
    metadata,
    treasuryHolder: treasuryHolder.publicKey,
    destination: claimToken.publicKey,
    vault: vault.publicKey,
    vaultOwnerBump,
    owner: vaultOwner,
  });

  const claimResourceRes = await transactionHandler.sendAndConfirmTransaction(
    claimResourceTx,
    [payer],
    defaultSendOptions,
  );

  assertConfirmedTransaction(t, claimResourceRes.txConfirmed);

  const createdToken = await getAccount(connection, claimToken.publicKey);

  t.assert(createdToken.mint.toBase58() === resourceMint.publicKey.toBase58());
  t.assert(createdToken.owner.toBase58() === payer.publicKey.toBase58());
});

test('claim resource:  should fail due to the treasury not empty', async (t) => {
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

  const startDate = Math.round(Date.now() / 1000) + 1;
  const params = {
    name: 'Market',
    description: '',
    startDate,
    endDate: null,
    mutable: true,
    price: 1,
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

  await sleep(3000);

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
    newTokenAccount: mintAta.publicKey,
  });

  const buyRes = await transactionHandler.sendAndConfirmTransaction(
    buyTx,
    [payer],
    defaultSendOptions,
  );

  logDebug('buy:: successful purchase');
  assertConfirmedTransaction(t, buyRes.txConfirmed);

  await sleep(3000);

  const marketTx = await closeMarket({
    transactionHandler,
    payer,
    connection,
    market,
  });

  const marketRes = await transactionHandler.sendAndConfirmTransaction(
    marketTx,
    [payer],
    defaultSendOptions,
  );

  logDebug(`market: ${market.publicKey}`);
  assertConfirmedTransaction(t, marketRes.txConfirmed);

  const metadata = await Metadata.getPDA(resourceMint.publicKey);

  const { tokenAccount: claimToken, createTokenTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: resourceMint.publicKey,
    connection,
  });

  const claimTokenRes = await transactionHandler.sendAndConfirmTransaction(
    createTokenTx,
    [claimToken],
    defaultSendOptions,
  );

  assertConfirmedTransaction(t, claimTokenRes.txConfirmed);

  const claimResourceTx = await createClaimResourceTransaction({
    connection,
    payer,
    market: market.publicKey,
    sellingResource: sellingResource.publicKey,
    metadata,
    treasuryHolder: treasuryHolder.publicKey,
    destination: claimToken.publicKey,
    vault: vault.publicKey,
    vaultOwnerBump,
    owner: vaultOwner,
  });

  try {
    await transactionHandler.sendAndConfirmTransaction(
      claimResourceTx,
      [payer],
      defaultSendOptions,
    );
  } catch (error) {
    logDebug(`expected transaction to fail due to 'treasury not empty'`);

    assertError(t, error, [/0x178a/i]);
  }
});
