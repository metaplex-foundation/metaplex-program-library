import test from 'tape';
import { Connection, Keypair } from '@solana/web3.js';
import {
  airdrop,
  assertConfirmedTransaction,
  defaultSendOptions,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman';

import { addLabel, connectionURL } from './utils';
import { mintNFT } from './actions/mint-nft';
import { findVaultOwnerAddress } from '../src/utils';
import { createTokenAccount } from './transactions/create-token-account';
import { createStoreTransaction } from './transactions/create-store';
import { createInitSellingResourceTransaction } from './transactions/init-selling-resouce';
import { buyMembershipToken } from './transactions/buy-membership-token';

test('buy-membership-token: success', async (t) => {
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
    [sellingResource, vault],
    defaultSendOptions,
  );

  addLabel('create:selling-resource', sellingResource);
  assertConfirmedTransaction(t, initSellingResourceRes.txConfirmed);

  const { tx: buyMemberShipTokenTx } = await buyMembershipToken({
    payer: payer.publicKey,
    connection,
    sellingResource: sellingResource.publicKey,
    vault: vault.publicKey,
    vaultOwner,
    vaultOwnerBump,
    resourceMint: resourceMint.publicKey,
    resourceMintMasterEdition: masterEdition,
  });

  const buyMemberShipTokenRes = await transactionHandler.sendAndConfirmTransaction(
    initSellingResourceTx,
    [],
    defaultSendOptions,
  );
  assertConfirmedTransaction(t, buyMemberShipTokenRes.txConfirmed);
});
