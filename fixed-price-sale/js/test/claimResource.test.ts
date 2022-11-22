import test from 'tape';
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore these exports actually exist but aren't setup correctly
import { getAccount, getAssociatedTokenAddress } from '@solana/spl-token';
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
import { CreateMarketInstructionArgs, TreasuryIsNotEmptyError } from '../src';
import { Metaplex, toBigNumber } from '@metaplex-foundation/js';

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
  const params: Omit<CreateMarketInstructionArgs, 'treasuryOwnerBump'> = {
    name: 'Market',
    description: '',
    startDate,
    endDate: null,
    mutable: true,
    price: 0.001,
    piecesInOneWallet: 1,
    gatingConfig: null,
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

  await sleep(3000);

  const marketTx = await closeMarket({
    transactionHandler,
    payer,
    connection,
    market,
  });

  await transactionHandler.sendAndConfirmTransaction(marketTx, [payer]).assertSuccess(t);

  logDebug(`market: ${market.publicKey}`);

  const [payoutTicket, payoutTicketBump] = await findPayoutTicketAddress(
    market.publicKey,
    payer.publicKey,
  );

  const destination = await getAssociatedTokenAddress(treasuryMint.publicKey, payer.publicKey);

  const metadata = await pdas.metadata({ mint: resourceMint.publicKey });

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

  await transactionHandler.sendAndConfirmTransaction(withdrawTx, [payer]).assertSuccess(t);

  const { tokenAccount: claimToken, createTokenTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: resourceMint.publicKey,
    connection,
  });

  await transactionHandler.sendAndConfirmTransaction(createTokenTx, [claimToken]).assertSuccess(t);

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

  await transactionHandler.sendAndConfirmTransaction(claimResourceTx, [payer]).assertSuccess(t);

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
  const params: Omit<CreateMarketInstructionArgs, 'treasuryOwnerBump'> = {
    name: 'Market',
    description: '',
    startDate,
    endDate: null,
    mutable: true,
    price: 1,
    piecesInOneWallet: 1,
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

  await sleep(3000);

  const marketTx = await closeMarket({
    transactionHandler,
    payer,
    connection,
    market,
  });

  await transactionHandler.sendAndConfirmTransaction(marketTx, [payer]).assertSuccess(t);

  logDebug(`market: ${market.publicKey}`);

  const metadata = await pdas.metadata({ mint: resourceMint.publicKey });

  const { tokenAccount: claimToken, createTokenTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: resourceMint.publicKey,
    connection,
  });

  await transactionHandler.sendAndConfirmTransaction(createTokenTx, [claimToken]).assertSuccess(t);

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
  await transactionHandler
    .sendAndConfirmTransaction(claimResourceTx, [payer])
    .assertError(t, TreasuryIsNotEmptyError);
  logDebug(`expected transaction to fail due to 'treasury not empty'`);
});
