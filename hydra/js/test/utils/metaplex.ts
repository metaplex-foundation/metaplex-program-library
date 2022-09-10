import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { DataV2, deprecated } from '@metaplex-foundation/mpl-token-metadata';
import BN from 'bn.js';
import * as spl from '@solana/spl-token';
// -----------------
// Create Metadata
// -----------------
// src/actions/createMetadata.ts

type CreateMetadataV2Params = {
  connection: Connection;
  publicKey: PublicKey;
  mint: PublicKey;
  metadataData: DataV2;
  updateAuthority?: PublicKey;
  payer: Keypair;
};

export async function createMetadataV2({
  connection,
  publicKey,
  mint,
  metadataData,
  updateAuthority,
  payer,
}: CreateMetadataV2Params) {
  const metadata = await deprecated.Metadata.getPDA(mint);
  const createMetadataTx = new deprecated.CreateMetadataV2(
    { feePayer: publicKey },
    {
      metadata,
      //@ts-ignore
      metadataData,
      updateAuthority: updateAuthority ?? publicKey,
      mint: mint,
      mintAuthority: publicKey,
    },
  );
  createMetadataTx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  await createMetadataTx.sign(payer);
  const createTxDetails = await connection.sendRawTransaction(createMetadataTx.serialize(), {
    skipPreflight: false,
  });

  return { metadata, createTxDetails };
}

// -----------------
// Prepare Mint and Create Metaata
// -----------------
export async function mintAndCreateMetadataV2(
  connection: Connection,
  payer: Keypair,
  args: DataV2,
) {
  const mint = await spl.Token.createMint(
    connection,
    payer,
    payer.publicKey,
    null,
    0,
    spl.TOKEN_PROGRAM_ID,
  );

  const fromTokenAccount = await mint.getOrCreateAssociatedAccountInfo(payer.publicKey);

  await mint.mintTo(fromTokenAccount.address, payer.publicKey, [], 1);
  const initMetadataData = args;
  const { metadata } = await createMetadataV2({
    connection,
    publicKey: payer.publicKey,
    mint: mint.publicKey,
    metadataData: initMetadataData,
    payer,
  });

  return { mint, metadata };
}

// -----------------
// Create A Master Edition
// -----------------
export async function createMasterEdition(
  connection: Connection,
  payer: Keypair,
  args: DataV2,
  maxSupply: number,
) {
  const { mint, metadata } = await mintAndCreateMetadataV2(connection, payer, args);

  const masterEditionPubkey = await deprecated.MasterEdition.getPDA(mint.publicKey);
  const createMev3 = new deprecated.CreateMasterEditionV3(
    { feePayer: payer.publicKey },
    {
      edition: masterEditionPubkey,
      metadata: metadata,
      updateAuthority: payer.publicKey,
      mint: mint.publicKey,
      mintAuthority: payer.publicKey,
      maxSupply: new BN(maxSupply),
    },
  );
  createMev3.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  await createMev3.sign(payer);

  const createTxDetails = await connection.sendRawTransaction(createMev3.serialize(), {
    skipPreflight: true,
  });
  await connection.confirmTransaction(createTxDetails, connection.commitment);
  return { mint, metadata, masterEditionPubkey, createTxDetails };
}
