import { PublicKey } from '@solana/web3.js';
import { defaultSendOptions, TransactionHandler } from '@metaplex-foundation/amman';
import {
  CreateMetadata,
  Metadata,
  MetadataDataData,
} from '@metaplex-foundation/mpl-token-metadata';

type CreateMetadataParams = {
  transactionHandler: TransactionHandler;
  publicKey: PublicKey;
  editionMint: PublicKey;
  metadataData: MetadataDataData;
};

export async function createMetadata({
  transactionHandler,
  publicKey,
  editionMint,
  metadataData,
}: CreateMetadataParams) {
  const metadata = await Metadata.getPDA(editionMint);
  const createMetadataTx = new CreateMetadata(
    { feePayer: publicKey },
    {
      metadata,
      metadataData,
      updateAuthority: publicKey,
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
