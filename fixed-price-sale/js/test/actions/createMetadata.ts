import { PublicKey } from '@solana/web3.js';
import { defaultSendOptions, TransactionHandler } from '@metaplex-foundation/amman';
import { deprecated } from '@metaplex-foundation/mpl-token-metadata';

const { CreateMetadata, Metadata } = deprecated

type CreateMetadataParams = {
  transactionHandler: TransactionHandler;
  publicKey: PublicKey;
  editionMint: PublicKey;
  metadataData: deprecated.MetadataDataData;
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
