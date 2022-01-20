import { Connection, PublicKey } from '@solana/web3.js';
import {
  Actions,
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
} from '@metaplex-foundation/mpl-token-metadata';
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { strict as assert } from 'assert';

import { createTokenAccount } from '../transactions/create-token-account';
import { createMetadata } from './create-metadata';
import { addLabel } from '../../test/utils';

type MintNFTParams = {
  transactionHandler: TransactionHandler;
  payer: PublicKey;
  connection: Connection;
};

const URI = 'https://arweave.net/Rmg4pcIv-0FQ7M7X838p2r592Q4NU63Fj7o7XsvBHEE';
const NAME = 'test';
const SYMBOL = 'sym';
const SELLER_FEE_BASIS_POINTS = 10;

export async function mintNFT({ transactionHandler, payer, connection }: MintNFTParams) {
  const { mint, createMintTx } = await new Actions(connection).createMintAccount(payer);
  const mintRes = await transactionHandler.sendAndConfirmTransaction(
    createMintTx,
    [mint],
    defaultSendOptions,
  );
  addLabel('create:mint', mint);
  assertConfirmedTransaction(assert, mintRes.txConfirmed);

  const { tokenAccount, createTokenTx } = await createTokenAccount({
    payer,
    mint: mint.publicKey,
    connection,
  });
  createTokenTx.add(
    Token.createMintToInstruction(
      new PublicKey(TOKEN_PROGRAM_ID),
      mint.publicKey,
      tokenAccount.publicKey,
      payer,
      [],
      1,
    ),
  );
  const associatedTokenAccountRes = await transactionHandler.sendAndConfirmTransaction(
    createTokenTx,
    [tokenAccount],
    defaultSendOptions,
  );
  addLabel('create:associated-token-account', mint);
  assertConfirmedTransaction(assert, associatedTokenAccountRes.txConfirmed);

  const initMetadataData = new MetadataDataData({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
  });

  const { createTxDetails, metadata } = await createMetadata({
    transactionHandler,
    publicKey: payer,
    editionMint: mint.publicKey,
    metadataData: initMetadataData,
  });
  addLabel('create:metadata', metadata);
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
    { feePayer: payer },
    {
      edition,
      metadata: metadataPDA,
      updateAuthority: payer,
      mint: mint.publicKey,
      mintAuthority: payer,
    },
  );

  const masterEditionRes = await transactionHandler.sendAndConfirmTransaction(masterEditionTx, [], {
    skipPreflight: false,
  });
  addLabel('create:master-edition', edition);
  assertConfirmedTransaction(assert, masterEditionRes.txConfirmed);

  return { tokenAccount, edition, editionBump, mint };
}
