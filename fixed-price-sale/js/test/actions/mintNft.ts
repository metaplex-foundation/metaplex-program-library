import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { PayerTransactionHandler } from '@metaplex-foundation/amman-client';
import {
  createCreateMasterEditionV3Instruction,
  Creator,
  DataV2,
  createCreateMetadataAccountV2Instruction,
} from '@metaplex-foundation/mpl-token-metadata';
import { createMintToInstruction } from '@solana/spl-token';
import { strict as assert } from 'assert';

import { createTokenAccount } from '../transactions/createTokenAccount';
import { CreateMint } from './createMintAccount';
import { Metaplex } from '@metaplex-foundation/js';

type MintNFTParams = {
  transactionHandler: PayerTransactionHandler;
  payer: Keypair;
  connection: Connection;
  maxSupply?: number;
  creators?: Creator[];
  collectionMint?: PublicKey;
};

const URI = 'https://arweave.net/Rmg4pcIv-0FQ7M7X838p2r592Q4NU63Fj7o7XsvBHEE';
const NAME = 'test';
const SYMBOL = 'sym';
const SELLER_FEE_BASIS_POINTS = 10;

export async function mintNFT({
  transactionHandler,
  payer,
  connection,
  creators,
  collectionMint,
  maxSupply = 100,
}: MintNFTParams) {
  const { mint, createMintTx } = await CreateMint.createMintAccount(connection, payer.publicKey);
  await transactionHandler.sendAndConfirmTransaction(createMintTx, [mint]).assertSuccess(assert);

  const { tokenAccount, createTokenTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: mint.publicKey,
    connection,
  });

  createTokenTx.add(
    createMintToInstruction(mint.publicKey, tokenAccount.publicKey, payer.publicKey, 1),
  );

  const data: DataV2 = {
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: creators ?? null,
    collection: collectionMint
      ? {
          key: collectionMint,
          verified: false,
        }
      : null,
    uses: null,
  };
  const metaplex = Metaplex.make(connection);
  const pdas = metaplex.nfts().pdas();
  const metadata = pdas.metadata({ mint: mint.publicKey });

  const createMetadataInstruction = createCreateMetadataAccountV2Instruction(
    {
      metadata,
      mint: mint.publicKey,
      updateAuthority: payer.publicKey,
      mintAuthority: payer.publicKey,
      payer: payer.publicKey,
    },
    { createMetadataAccountArgsV2: { isMutable: true, data } },
  );

  createTokenTx.add(createMetadataInstruction);

  const edition = pdas.edition({ mint: mint.publicKey });

  const masterEditionInstruction = createCreateMasterEditionV3Instruction(
    {
      edition,
      metadata,
      updateAuthority: payer.publicKey,
      mint: mint.publicKey,
      mintAuthority: payer.publicKey,
      payer: payer.publicKey,
    },
    {
      createMasterEditionArgs: { maxSupply },
    },
  );

  createTokenTx.add(masterEditionInstruction);

  await transactionHandler
    .sendAndConfirmTransaction(createTokenTx, [tokenAccount])
    .assertSuccess(assert);

  return { tokenAccount, edition, editionBump: edition.bump, mint, metadata };
}
