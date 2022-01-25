import test from 'tape';
import { Connection, Keypair } from '@solana/web3.js';
import { connectionURL, killStuckProcess } from './utils';
import {
  airdrop,
  assertConfirmedTransaction,
  PayerTransactionHandler,
  defaultSendOptions,
} from '@metaplex-foundation/amman';

import { findVaultOwnerAddress } from '../src/utils';

import { addLabel, logDebug } from './utils';
import { createStoreTransaction } from './transactions/create-store';
import { mintNFT } from './actions/mint-nft';
import { createInitSellingResourceTransaction } from './transactions/init-selling-resouce';
import { createTokenAccount } from './transactions/create-token-account';

killStuckProcess();

test('init-selling-resource: success', async (t) => {
  const payer = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const { store, transaction: createStoreTx } = await createStoreTransaction(payer, connection);

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
  logDebug(`mint: ${resourceMint.publicKey}`);

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
  const initSellingResourceRes = await transactionHandler.sendAndConfirmTransaction(
    initSellingResourceTx,
    [sellingResource],
    defaultSendOptions,
  );

  addLabel('create:selling-resource', sellingResource);
  assertConfirmedTransaction(t, initSellingResourceRes.txConfirmed);

  logDebug(`sellingResource: ${store.publicKey}`);
  logDebug(initSellingResourceRes.txSummary.logMessages.join('\n'));
});
