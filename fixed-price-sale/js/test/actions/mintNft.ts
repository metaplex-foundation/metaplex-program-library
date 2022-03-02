import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import {
  defaultSendOptions,
  TransactionHandler,
  assertConfirmedTransaction,
} from '@metaplex-foundation/amman';
import {
  MetadataDataData,
  MetadataProgram,
  MasterEdition,
  Metadata,
  CreateMasterEdition,
  Creator,
} from '@metaplex-foundation/mpl-token-metadata';
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { strict as assert } from 'assert';

import { createTokenAccount } from '../transactions/createTokenAccount';
import { createMetadata } from './createMetadata';
import { CreateMint } from './createMintAccount';

type MintNFTParams = {
  transactionHandler: TransactionHandler;
  payer: Keypair;
  connection: Connection;
  creators?: Creator[];
};

const URI = 'https://arweave.net/Rmg4pcIv-0FQ7M7X838p2r592Q4NU63Fj7o7XsvBHEE';
const NAME = 'test';
const SYMBOL = 'sym';
const SELLER_FEE_BASIS_POINTS = 10;

export async function mintNFT({ transactionHandler, payer, connection, creators }: MintNFTParams) {
  const { mint, createMintTx } = await CreateMint.createMintAccount(connection, payer.publicKey);
  const mintRes = await transactionHandler.sendAndConfirmTransaction(
    createMintTx,
    [mint],
    defaultSendOptions,
  );
  assertConfirmedTransaction(assert, mintRes.txConfirmed);

  const { tokenAccount, createTokenTx } = await createTokenAccount({
    payer: payer.publicKey,
    mint: mint.publicKey,
    connection,
  });

  createTokenTx.add(
    Token.createMintToInstruction(
      new PublicKey(TOKEN_PROGRAM_ID),
      mint.publicKey,
      tokenAccount.publicKey,
      payer.publicKey,
      [],
      1,
    ),
  );

  const associatedTokenAccountRes = await transactionHandler.sendAndConfirmTransaction(
    createTokenTx,
    [tokenAccount],
    defaultSendOptions,
  );
  assertConfirmedTransaction(assert, associatedTokenAccountRes.txConfirmed);

  const initMetadataData = new MetadataDataData({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: creators ?? null,
  });

  const { createTxDetails } = await createMetadata({
    transactionHandler,
    publicKey: payer.publicKey,
    editionMint: mint.publicKey,
    metadataData: initMetadataData,
  });
  assertConfirmedTransaction(assert, createTxDetails.txConfirmed);

  const metadataPDA = await Metadata.getPDA(mint.publicKey);
  const [edition, editionBump] = await PublicKey.findProgramAddress(
    [
      Buffer.from(MetadataProgram.PREFIX),
      MetadataProgram.PUBKEY.toBuffer(),
      new PublicKey(mint.publicKey).toBuffer(),
      Buffer.from(MasterEdition.EDITION_PREFIX),
    ],
    MetadataProgram.PUBKEY,
  );

  const masterEditionTx = new CreateMasterEdition(
    { feePayer: payer.publicKey },
    {
      edition,
      metadata: metadataPDA,
      updateAuthority: payer.publicKey,
      mint: mint.publicKey,
      mintAuthority: payer.publicKey,
    },
  );

  const masterEditionRes = await transactionHandler.sendAndConfirmTransaction(masterEditionTx, [], {
    skipPreflight: false,
  });
  assertConfirmedTransaction(assert, masterEditionRes.txConfirmed);

  return { tokenAccount, edition, editionBump, mint, metadata: metadataPDA };
}
