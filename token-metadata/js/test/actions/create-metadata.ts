import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { addLabel, logDebug } from '../utils';
import {
  Actions,
  assertConfirmedTransaction,
  defaultSendOptions,
  TransactionHandler,
} from '@metaplex-foundation/amman';
import { strict as assert } from 'assert';
import { CreateMetadata, TokenData, Data } from '../../src/mpl-token-metadata';

// -----------------
// Create Metadata
// -----------------
// src/actions/createMetadata.ts
type CreateMetadataParams = {
  transactionHandler: TransactionHandler;
  publicKey: PublicKey;
  editionMint: PublicKey;
  data: Data;
  updateAuthority?: PublicKey;
};

export async function createMetadata({
  transactionHandler,
  publicKey,
  editionMint,
  data,
  updateAuthority,
}: CreateMetadataParams) {
  const metadata = await TokenData.getPDA(editionMint);
  const createMetadataTx = new CreateMetadata(
    { feePayer: publicKey },
    {
      pubkey: publicKey,
      data,
      updateAuthority: updateAuthority ?? publicKey,
      mint: editionMint,
      mintAuthority: publicKey,
    },
  );

  const createTxDetails = await transactionHandler.sendAndConfirmTransaction(
    createMetadataTx,
    [],
    defaultSendOptions,
  );

  return { metadata, createTxDetails };
}

// -----------------
// Prepare Mint and Create Metaata
// -----------------
export async function mintAndCreateMetadata(
  connection: Connection,
  transactionHandler: TransactionHandler,
  payer: Keypair,
  args: ConstructorParameters<typeof Data>[0],
) {
  const { createMintAccount } = new Actions(connection);
  const { mint, createMintTx } = await createMintAccount(payer.publicKey);
  const mintRes = await transactionHandler.sendAndConfirmTransaction(
    createMintTx,
    [mint],
    defaultSendOptions,
  );
  addLabel('mint', mint);

  assertConfirmedTransaction(assert, mintRes.txConfirmed);

  const initData = new Data(args);

  const { createTxDetails, metadata } = await createMetadata({
    transactionHandler,
    publicKey: payer.publicKey,
    editionMint: mint.publicKey,
    data: initData,
  });

  addLabel('metadata', metadata);
  logDebug(createTxDetails.txSummary.logMessages.join('\n'));

  return { mint, metadata };
}
