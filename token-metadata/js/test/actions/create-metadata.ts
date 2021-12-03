import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import {
  addLabel,
  assertConfirmedTransaction,
  defaultSendOptions,
  logDebug,
  TransactionHandler,
} from '../utils';
import { createMintAccount } from './create-mint';
import { CreateMetadata, Metadata, MetadataDataData } from '../../';
import { strict as assert } from 'assert';

// -----------------
// Create Metadata
// -----------------
// src/actions/createMetadata.ts
type CreateMetadataParams = {
  transactionHandler: TransactionHandler;
  publicKey: PublicKey;
  editionMint: PublicKey;
  metadataData: MetadataDataData;
  updateAuthority?: PublicKey;
};

export async function createMetadata({
  transactionHandler,
  publicKey,
  editionMint,
  metadataData,
  updateAuthority,
}: CreateMetadataParams) {
  const metadata = await Metadata.getPDA(editionMint);
  const createMetadataTx = new CreateMetadata(
    { feePayer: publicKey },
    {
      metadata,
      metadataData,
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
  args: ConstructorParameters<typeof MetadataDataData>[0],
) {
  const { mint, createMintTx } = await createMintAccount(connection, payer.publicKey);
  const mintRes = await transactionHandler.sendAndConfirmTransaction(
    createMintTx,
    [mint],
    defaultSendOptions,
  );
  addLabel('mint', mint);

  assertConfirmedTransaction(assert, mintRes.txConfirmed);

  const initMetadataData = new MetadataDataData(args);

  const { createTxDetails, metadata } = await createMetadata({
    transactionHandler,
    publicKey: payer.publicKey,
    editionMint: mint.publicKey,
    metadataData: initMetadataData,
  });

  addLabel('metadata', metadata);
  logDebug(createTxDetails.txSummary.logMessages.join('\n'));

  return { mint, metadata };
}
