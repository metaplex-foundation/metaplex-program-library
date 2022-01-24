import BN from 'bn.js';
import test from 'tape';
import {
  Actions,
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
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { strict as assert } from 'assert';

test('buy: success', async (t) => {
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
    edition: resourceMasterEdition,
    editionBump: resourceMasterEditionBump,
    tokenAccount: resourceToken,
    mint: resourceMint,
  } = await mintNFT({
    transactionHandler,
    payer: payer.publicKey,
    connection,
  });

  // const resourceMintMetadata = await Metadata.getPDA(resourceMint.publicKey);
  //
  // const resourceMintMasterMetadata = await Metadata.getPDA(resourceMasterEdition);
  //
  // const resourceMintEdition = await Edition.getPDA(resourceMint.publicKey);
  //
  // const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));

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
    masterEdition: resourceMasterEdition,
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

  // const {
  //   mint: treasuryMint,
  //   edition: treasuryMintMasterEdition,
  //   tokenAccount: treasuryToken,
  // } = await mintNFT({
  //   transactionHandler,
  //   payer: payer.publicKey,
  //   connection,
  // });

  const treasuryMint = new PublicKey('11111111111111111111111111111111');

  addLabel('create:market-treasury-mint', treasuryMint);

  // const treasuryMintMetadata = await Metadata.getPDA(treasuryMint.publicKey);
  //
  // const treasuryMintEdition = await Edition.getPDA(treasuryMint.publicKey);
  //
  // const treasuryMintMasterMetadata = await Metadata.getPDA(treasuryMintMasterEdition);
  //
  // const treasuryMintEditionMarker = await EditionMarker.getPDA(treasuryMint.publicKey, new BN(1));

  const [treasuryOwner, treasuryOwnerBump] = await findTresuryOwnerAddress(
    treasuryMint,
    sellingResource.publicKey,
  );

  addLabel('get:market-treasury-owner', treasuryOwner);

  // const { tokenAccount: treasuryHolder, createTokenTx: createTreasuryTx } =
  //   await createTokenAccount({
  //     payer: payer.publicKey,
  //     connection,
  //     mint: treasuryMint,
  //     owner: treasuryOwner,
  //   });

  // const createTreasuryRes = await transactionHandler.sendAndConfirmTransaction(
  //   createTreasuryTx,
  //   [treasuryHolder],
  //   defaultSendOptions,
  // );
  //
  // logDebug('buy:: created market treasury holder', treasuryHolder.publicKey.toBase58());
  // addLabel('create:market-treasury-holder', treasuryHolder.publicKey.toBase58());
  // assertConfirmedTransaction(t, createTreasuryRes.txConfirmed);

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

  // const { mint: newMint, edition: newMintMasterEdition } = await mintNFT({
  //   transactionHandler,
  //   payer: payer.publicKey,
  //   connection,
  // });

  const { mint: newMint, createMintTx } = await new Actions(connection).createMintAccount(
    payer.publicKey,
  );
  await transactionHandler.sendAndConfirmTransaction(createMintTx, [newMint], defaultSendOptions);

  const { tokenAccount, createTokenTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: newMint.publicKey,
    connection,
  });
  createTokenTx.add(
    Token.createMintToInstruction(
      new PublicKey(TOKEN_PROGRAM_ID),
      newMint.publicKey,
      tokenAccount.publicKey,
      payer.publicKey,
      [],
      1,
    ),
  );
  await transactionHandler.sendAndConfirmTransaction(
    createTokenTx,
    [tokenAccount],
    defaultSendOptions,
  );

  const newMintMetadata = await Metadata.getPDA(newMint.publicKey);
  //
  const newMintEdition = await Edition.getPDA(newMint.publicKey);
  //
  const resourceMintMasterMetadata = await Metadata.getPDA(resourceMasterEdition);
  //
  const resourceMintEditionMarker = await EditionMarker.getPDA(resourceMint.publicKey, new BN(1));

  const { tx: buyTx } = await createBuyTransaction({
    connection,
    buyer: payer.publicKey,
    buyerTokenAccount: payer.publicKey,
    sellingResource: sellingResource.publicKey,
    market: market.publicKey,
    marketTreasuryHolder: treasuryOwner,
    tradeHistory,
    tradeHistoryBump,
    vault: vault.publicKey,
    vaultOwnerBump,
    treasuryOwner,
    // newMint: resourceMint.publicKey,
    // newMintEdition: resourceMintEdition,
    // newMintMetadata: resourceMintMetadata,
    // newMintEditionMarker: resourceMintEditionMarker,
    // newMintMasterEdition: resourceMasterEdition,
    // newMintMasterMetadata: resourceMintMasterMetadata,
    // newMint: treasuryMint.publicKey, // ?
    // newMintEdition: treasuryMintEdition,
    // newMintMetadata: treasuryMintMetadata,
    // newMintEditionMarker: treasuryMintEditionMarker,
    // newMintMasterEdition: treasuryMintMasterEdition,
    // newMintMasterMetadata: treasuryMintMasterMetadata,
    newMint: newMint.publicKey,
    newMintEdition,
    newMintMetadata,
    newMintEditionMarker: resourceMintEditionMarker,
    newMintMasterEdition: resourceMasterEdition,
    newMintMasterMetadata: resourceMintMasterMetadata,
  });

  const buyRes = await transactionHandler.sendAndConfirmTransaction(
    buyTx,
    [payer],
    defaultSendOptions,
  );

  logDebug('buy:: successful purchase');
  assertConfirmedTransaction(t, buyRes.txConfirmed);
});
