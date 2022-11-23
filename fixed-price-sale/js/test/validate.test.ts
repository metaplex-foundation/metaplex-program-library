import test from 'tape';
import { findTradeHistoryAddress, validateMembershipToken } from '../src/utils';
import { createBuyTransaction } from './transactions';
import { killStuckProcess, logDebug, sleep } from './utils';
import {
  createPrerequisites,
  createStore,
  initSellingResource,
  mintTokenToAccount,
  mintNFT,
  createMarket,
} from './actions';
import { CreateMarketInstructionArgs } from '../src';
import { Account as TokenAccount, getAccount } from '@solana/spl-token';
import { Metaplex, toBigNumber } from '@metaplex-foundation/js';

killStuckProcess();

test('validate: successful purchase and validation', async (t) => {
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

  const { mint: newMint, mintAta: newMintAta } = await mintTokenToAccount({
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
    newTokenAccount: newMintAta.publicKey,
  });

  await transactionHandler.sendAndConfirmTransaction(buyTx, [payer]).assertSuccess(t);
  logDebug('validate: successful purchase');

  console.log(resourceMintMasterEdition.toString(), userTokenAcc.publicKey.toString());

  const ta = await getAccount(connection, newMintAta.publicKey);
  const result = await validateMembershipToken(connection, resourceMintMasterEdition, ta);

  logDebug('validate: copy is valid');
  t.equal(result, true);
});

test('validate: successful purchase and failed validation', async (t) => {
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

  const { mint: newMint, mintAta: newMintAta } = await mintTokenToAccount({
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
    newTokenAccount: newMintAta.publicKey,
  });

  await transactionHandler.sendAndConfirmTransaction(buyTx, [payer]).assertSuccess(t);
  logDebug('validate: successful purchase');

  const { edition: masterEdition } = await mintNFT({
    transactionHandler,
    payer,
    connection,
  });

  const ta: TokenAccount = await getAccount(connection, newMintAta.publicKey);
  const result = await validateMembershipToken(connection, masterEdition, ta);

  logDebug('validate: copy is invalid');
  t.equal(result, false);
});
