import BN from 'bn.js';
import test from 'tape';
import { assertConfirmedTransaction, defaultSendOptions } from '@metaplex-foundation/amman';
import {
  Edition,
  EditionMarker,
  MasterEdition,
  Metadata,
} from '@metaplex-foundation/mpl-token-metadata';
import { TokenAccount } from '@metaplex-foundation/mpl-core';

import { findTradeHistoryAddress, validateMembershipToken } from '../src/utils';
import { createBuyTransaction } from './transactions';
import { killStuckProcess, logDebug, sleep } from './utils';
import {
  mintNFT,
  mintTokenToAccount,
  createMarket,
  createPrerequisites,
  createStore,
  initSellingResource,
} from './actions';

killStuckProcess();

test('validate: successful purchase and validation', async (t) => {
  const { payer, connection, transactionHandler } = await createPrerequisites();
  console.log("🚀 ~ file: validate.test.ts ~ line 28 ~ test ~ transactionHandler -  payer", transactionHandler["payer"].publicKey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 28 ~ test ~ connection - rpcendpoint", connection["_rpcEndpoint"])
  console.log("🚀 ~ file: validate.test.ts ~ line 28 ~ test ~ payer", payer.publicKey.toBase58())

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
  console.log("🚀 ~ file: validate.test.ts ~ line 42 ~ test ~ store", store.publicKey.toBase58())

  const { sellingResource, vault, vaultOwner, vaultOwnerBump, resourceMint } =
    await initSellingResource({
      test: t,
      transactionHandler,
      payer,
      connection,
      store: store.publicKey,
      maxSupply: 100,
    });
  console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ resourceMint", resourceMint.publicKey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ vaultOwnerBump", vaultOwnerBump)
  console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ vaultOwner", vaultOwner.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ vault", vault.publicKey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ sellingResource", sellingResource.publicKey.toBase58())

  const { mint: treasuryMint, tokenAccount: userTokenAcc } = await mintNFT({
    transactionHandler,
    payer,
    connection,
  });
  console.log("🚀 ~ file: validate.test.ts ~ line 60 ~ test ~ treasuryMint", treasuryMint.publicKey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 65 ~ test ~ userTokenAcc", userTokenAcc.publicKey.toBase58())

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
  console.log("🚀 ~ file: validate.test.ts ~ line 79 ~ test ~ treasuryHolder", treasuryHolder.publicKey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 89 ~ test ~ market", market.publicKey.toBase58())

  const [tradeHistory, tradeHistoryBump] = await findTradeHistoryAddress(
    payer.publicKey,
    market.publicKey,
  );
  console.log("🚀 ~ file: validate.test.ts ~ line 92 ~ test ~ tradeHistoryBump", tradeHistoryBump)
  console.log("🚀 ~ file: validate.test.ts ~ line 95 ~ test ~ tradeHistory", tradeHistory.toBase58())

  const { mint: newMint, mintAta: newMintAta } = await mintTokenToAccount({
    connection,
    payer: payer.publicKey,
    transactionHandler,
  });
  console.log("🚀 ~ file: validate.test.ts ~ line 99 ~ test ~ newMint", newMint.publicKey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 104 ~ test ~ newMintAta", newMintAta.publicKey.toBase58())

  logDebug('new mint', newMint.publicKey.toBase58());

  const newMintEdition = await Edition.getPDA(newMint.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 109 ~ test ~ newMintEdition", newMintEdition.toBase58())
  const newMintMetadata = await Metadata.getPDA(newMint.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 111 ~ test ~ newMintMetadata", newMintMetadata.toBase58())

  const resourceMintMasterEdition = await Edition.getPDA(resourceMint.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 114 ~ test ~ resourceMintMasterEdition", resourceMintMasterEdition.toBase58())
  const resourceMintMetadata = await Metadata.getPDA(resourceMint.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 116 ~ test ~ resourceMintMetadata", resourceMintMetadata.toBase58())
  const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));
  console.log("🚀 ~ file: validate.test.ts ~ line 118 ~ test ~ resourceMintEditionMarker", resourceMintEditionMarker.toBase58())

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

  console.log("🚀 ~ file: validate.test.ts ~ line 123 ~ test ~ buyTx", buyTx)
  const buyRes = await transactionHandler.sendAndConfirmTransaction(
    buyTx,
    [payer],
    defaultSendOptions,
  );
  console.log("🚀 ~ file: validate.test.ts ~ line 148 ~ test ~ buyRes", buyRes["txSignature"])

  logDebug('validate: successful purchase');
  assertConfirmedTransaction(t, buyRes.txConfirmed);

  const me = await MasterEdition.load(connection, resourceMintMasterEdition);
  console.log("🚀 ~ file: validate.test.ts ~ line 154 ~ test ~ MasterEdition", me.pubkey.toBase58())
  console.log("Built in console: ",
    "Master Edition me: ", me.pubkey.toString(),
    "resourceMintMasterEdition: ",resourceMintMasterEdition.toString(),
    "userTokenAcc: ", userTokenAcc.publicKey.toString(),
  );

  const ta = await TokenAccount.load(connection, newMintAta.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 161 ~ test ~ TokenAccount", ta.pubkey.toBase58())
  const result = await validateMembershipToken(connection, me, ta);
  console.log("🚀 ~ file: validate.test.ts ~ line 163 ~ test ~ result", result)

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
  console.log("🚀 ~ file: validate.test.ts ~ line 183 ~ test2 ~ store", store.publicKey.toBase58())

  const { sellingResource, vault, vaultOwner, vaultOwnerBump, resourceMint } =
    await initSellingResource({
      test: t,
      transactionHandler,
      payer,
      connection,
      store: store.publicKey,
      maxSupply: 100,
    });
    console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ resourceMint", resourceMint.publicKey.toBase58())
    console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ vaultOwnerBump", vaultOwnerBump)
    console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ vaultOwner", vaultOwner.toBase58())
    console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ vault", vault.publicKey.toBase58())
    console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ sellingResource", sellingResource.publicKey.toBase58())

  const { mint: treasuryMint, tokenAccount: userTokenAcc } = await mintNFT({
    transactionHandler,
    payer,
    connection,
  });
  console.log("🚀 ~ file: validate.test.ts ~ line 201 ~ test2 ~ treasuryMint", treasuryMint.publicKey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 206 ~ test2 ~ userTokenAcc", userTokenAcc.publicKey.toBase58())

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
  console.log("🚀 ~ file: validate.test.ts ~ line 220 ~ test2 ~ market", market.publicKey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 220 ~ test2 ~ treasuryHolder", treasuryHolder.publicKey.toBase58())

  const [tradeHistory, tradeHistoryBump] = await findTradeHistoryAddress(
    payer.publicKey,
    market.publicKey,
  );
  console.log("🚀 ~ file: validate.test.ts ~ line 237 ~ test2 ~ tradeHistory", tradeHistory.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 233 ~ test2 ~ tradeHistoryBump", tradeHistoryBump)


  const { mint: newMint, mintAta: newMintAta } = await mintTokenToAccount({
    connection,
    payer: payer.publicKey,
    transactionHandler,
  });
  console.log("🚀 ~ file: validate.test.ts ~ line 241 ~ test2 ~ newMint", newMint.publicKey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 246 ~ test2 ~ newMintAta", newMintAta.publicKey.toBase58())

  logDebug('new mint', newMint.publicKey.toBase58());

  const newMintEdition = await Edition.getPDA(newMint.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 251 ~ test2 ~ newMintEdition", newMintEdition.toBase58())
  const newMintMetadata = await Metadata.getPDA(newMint.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 253 ~ test2 ~ newMintMetadata", newMintMetadata.toBase58())

  const resourceMintMasterEdition = await Edition.getPDA(resourceMint.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 256 ~ test2 ~ resourceMintMasterEdition", resourceMintMasterEdition.toBase58())
  const resourceMintMetadata = await Metadata.getPDA(resourceMint.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 257 ~ test2 ~ resourceMintMetadata", resourceMintMetadata.toBase58())
  const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));
  console.log("🚀 ~ file: validate.test.ts ~ line 260 ~ test2 ~ resourceMintEditionMarker", resourceMintEditionMarker.toBase58())

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
  console.log("🚀 ~ file: validate.test.ts ~ line 265 ~ test2 ~ buyTx", buyTx)


  const buyRes = await transactionHandler.sendAndConfirmTransaction(
    buyTx,
    [payer],
    defaultSendOptions,
  );
  console.log("🚀 ~ file: validate.test.ts ~ line 291 ~ test2 ~ buyRes", buyRes.txSignature)

  logDebug('validate: successful purchase');
  assertConfirmedTransaction(t, buyRes.txConfirmed);

  const { edition: masterEdition } = await mintNFT({
    transactionHandler,
    payer,
    connection,
  });
  console.log("🚀 ~ file: validate.test.ts ~ line 297 ~ test2 ~ masterEdition", masterEdition.toBase58())

  const me = await MasterEdition.load(connection, masterEdition);
  console.log("🚀 ~ file: validate.test.ts ~ line 280 ~ test2 ~ me", me.pubkey.toBase58())
  console.log("🚀 ~ file: validate.test.ts ~ line 280 ~ test2 ~ masterEdition", masterEdition.toBase58())
  
  const ta = await TokenAccount.load(connection, newMintAta.publicKey);
  console.log("🚀 ~ file: validate.test.ts ~ line 308 ~ test2 ~ ta", ta.pubkey.toBase58())
  const result = await validateMembershipToken(connection, me, ta);
  console.log("🚀 ~ file: validate.test.ts ~ line 310 ~ test2 ~ result", result)

  logDebug('validate: copy is invalid');
  t.equal(result, false);
});
