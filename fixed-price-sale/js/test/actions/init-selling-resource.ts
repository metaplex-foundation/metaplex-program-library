import test from 'tape';
import {
  assertConfirmedTransaction,
  defaultSendOptions,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';

import { findVaultOwnerAddress } from '../../src/utils';

import { createAndSignTransaction, logDebug } from '../utils';
import { createTokenAccount } from '../transactions/create-token-account';
import { mintNFT } from './mint-nft';
import { createInitSellingResourceInstruction } from '../../src/instructions';
import { Creator } from '@metaplex-foundation/mpl-token-metadata';

type InitSellingResourceParams = {
  test: test.Test;
  transactionHandler: PayerTransactionHandler;
  payer: Keypair;
  connection: Connection;
  store: PublicKey;
  maxSupply: number | null;
};

export const initSellingResource = async ({
  test,
  transactionHandler,
  payer,
  connection,
  store,
  maxSupply,
}: InitSellingResourceParams): Promise<{
  sellingResource: Keypair;
  vault: Keypair;
  vaultOwner: PublicKey;
  vaultOwnerBump: number;
  resourceMint: Keypair;
}> => {
  const creator = new Creator({
    address: payer.publicKey.toBase58(),
    share: 100,
    verified: true,
  });

  const {
    edition: masterEdition,
    editionBump: masterEditionBump,
    tokenAccount: resourceToken,
    mint: resourceMint,
    metadata,
  } = await mintNFT({
    transactionHandler,
    payer,
    connection,
    creators: [creator],
  });

  const [vaultOwner, vaultOwnerBump] = await findVaultOwnerAddress(resourceMint.publicKey, store);
  const { tokenAccount: vault, createTokenTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: resourceMint.publicKey,
    connection,
    owner: vaultOwner,
  });
  const createVaultRes = await transactionHandler.sendAndConfirmTransaction(
    createTokenTx,
    [vault],
    defaultSendOptions,
  );
  assertConfirmedTransaction(test, createVaultRes.txConfirmed);

  const sellingResource = Keypair.generate();

  const initSellingResourceInstruction = createInitSellingResourceInstruction(
    {
      store,
      admin: payer.publicKey,
      sellingResource: sellingResource.publicKey,
      sellingResourceOwner: payer.publicKey,
      metadata,
      masterEdition,
      resourceMint: resourceMint.publicKey,
      resourceToken: resourceToken.publicKey,
      vault: vault.publicKey,
      owner: vaultOwner,
    },
    {
      masterEditionBump,
      vaultOwnerBump,
      maxSupply,
    },
  );

  const initSellingResourceTx = await createAndSignTransaction(
    connection,
    payer,
    [initSellingResourceInstruction],
    [sellingResource],
  );

  const initSellingResourceRes = await transactionHandler.sendAndConfirmTransaction(
    initSellingResourceTx,
    [sellingResource],
    defaultSendOptions,
  );

  logDebug(`selling-resource: ${sellingResource.publicKey}`);
  assertConfirmedTransaction(test, initSellingResourceRes.txConfirmed);

  return {
    sellingResource,
    vault,
    vaultOwner,
    vaultOwnerBump,
    resourceMint,
  };
};
