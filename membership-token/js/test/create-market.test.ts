import test from 'tape';
import { Connection, Keypair } from '@solana/web3.js';
import { connectionURL, killStuckProcess } from './utils';
import {
  airdrop,
  assertConfirmedTransaction,
  PayerTransactionHandler,
  defaultSendOptions,
} from '@metaplex-foundation/amman';

import { findTresuryOwnerAddress, findVaultOwnerAddress } from '../src/utils';

import {
  createMarketTransaction,
  createTokenAccount,
  createInitSellingResourceTransaction,
  createStoreTransaction,
} from './transactions';
import { addLabel, logDebug } from './utils';
import { mintNFT } from './actions/mint-nft';

killStuckProcess();

test('create-market: success', async (t) => {
  const payer = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 20);
  const { store, transaction: createStoreTx } = await createStoreTransaction(payer, connection);
  logDebug('STORE store: ', store.publicKey.toBase58());
  logDebug('STORE payer: ', payer.publicKey.toBase58());

  const createStoreRes = await transactionHandler.sendAndConfirmTransaction(
    createStoreTx,
    [store],
    defaultSendOptions,
  );
  addLabel('create:store', store);
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
  logDebug(`SR masterEdition:     ${masterEdition.toBase58()}`);
  logDebug(`SR masterEditionBump: ${masterEditionBump}`);
  logDebug(`SR resourceToken:     ${resourceToken.publicKey.toBase58()}`);
  logDebug(`SR resourceMint:      ${resourceMint.publicKey.toBase58()}`);

  const [vaultOwner, vaultOwnerBump] = await findVaultOwnerAddress(
    resourceMint.publicKey,
    store.publicKey,
  );

  logDebug('-- Vault Owner -------------------------------');
  logDebug(`SR vaultOwner:        ${vaultOwner.toBase58()}`);
  logDebug(`SR vaultOwnerBump:    ${vaultOwnerBump}`);

  const { tokenAccount: vault, createTokenTx: createVaultTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: resourceMint.publicKey,
    connection,
    owner: vaultOwner,
  });

  logDebug('-- Vault  -------------------------------');
  logDebug(`SR vault:             ${vault.publicKey.toBase58()}`);
  logDebug(`SR createVaultTx:     ${createVaultTx}`);
  const createVaultRes = await transactionHandler.sendAndConfirmTransaction(
    createVaultTx,
    [vault],
    defaultSendOptions,
  );
  addLabel('create:vault', vault);
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

  logDebug('-- SR  -------------------------------');
  logDebug(`SR sellingResource:   ${sellingResource.publicKey.toBase58()}`);

  const initSellingResourceRes = await transactionHandler.sendAndConfirmTransaction(
    initSellingResourceTx,
    [sellingResource],
    defaultSendOptions,
  );

  addLabel('create:selling-resource', sellingResource);
  assertConfirmedTransaction(t, initSellingResourceRes.txConfirmed);

  logDebug('-- initSellingResourceRes  --------------');
  logDebug(initSellingResourceRes.txSummary.logMessages.join('\n'));

  const { mint: treasuryMint } = await mintNFT({
    transactionHandler,
    payer: payer.publicKey,
    connection,
  });

  logDebug('-- Market  -------------------------------');
  logDebug(`Market treasuryMint:   ${treasuryMint.publicKey.toBase58()}`);

  const [treasuryOwner, treasuryOwnerBump] = await findTresuryOwnerAddress(
    treasuryMint.publicKey,
    sellingResource.publicKey,
  );

  logDebug(`Market treasuryOwner:       ${treasuryOwner.toBase58()}`);
  logDebug(`Market treasuryOwnerBump:   ${treasuryOwnerBump}`);

  const { tokenAccount: treasuryHolder, createTokenTx: createTreasuryTx } =
    await createTokenAccount({
      payer: payer.publicKey,
      connection,
      mint: treasuryMint.publicKey,
      owner: treasuryOwner,
    });

  logDebug(`Market treasuryHolder:   ${treasuryHolder.publicKey.toBase58()}`);
  const createTreasuryRes = await transactionHandler.sendAndConfirmTransaction(
    createTreasuryTx,
    [treasuryHolder],
    defaultSendOptions,
  );
  addLabel('create:treasury', treasuryHolder);
  assertConfirmedTransaction(t, createTreasuryRes.txConfirmed);
  const startDate = Math.round(Date.now() / 1000) + 5;

  const endDate = startDate + 5 * 20;

  const marketName = 'N';
  const marketDescription = 'D';

  const mutable = true;
  const price = 2;
  const piecesInOneWallet = 10;

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
    name: marketName,
    description: marketDescription,
    mutable,
    price,
    piecesInOneWallet,
    startDate,
    endDate,
  });

  const marketRes = await transactionHandler.sendAndConfirmTransaction(
    marketTx,
    [market, payer],
    defaultSendOptions,
  );

  addLabel('create:market', market);
  assertConfirmedTransaction(t, marketRes.txConfirmed);

  logDebug(`market: ${market.publicKey}`);
  logDebug(marketRes.txSummary.logMessages.join('\n'));
});
