import test from 'tape';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { CreateMetadata, Edition, EditionData, Metadata, MetadataDataData } from '../';
import {
  TransactionHandler,
  connectionURL,
  airdrop,
  PayerTransactionHandler,
  dump,
  killStuckProcess,
  defaultSendOptions,
} from './utils';
import { createMintAccount } from './utils/CreateMint';
import { assertConfirmedTransaction, assertTransactionSummary } from './utils/asserts';

import BN from 'bn.js';

import { logDebug } from './utils';

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

async function createMetadata({
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

  return transactionHandler.sendAndConfirmTransaction(createMetadataTx, [], defaultSendOptions);
}

killStuckProcess();

const URI = 'uri';
const NAME = 'test';
const SYMBOL = 'sym';

test('create-metadata-account: success', async (t) => {
  const payer = Keypair.generate();
  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const { mint, createMintTx } = await createMintAccount(connection, payer.publicKey);
  const mintRes = await transactionHandler.sendAndConfirmTransaction(
    createMintTx,
    [mint],
    defaultSendOptions,
  );

  assertConfirmedTransaction(t, mintRes.txConfirmed);

  const metadata = new MetadataDataData({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: 10,
    creators: null,
  });

  const createRes = await createMetadata({
    transactionHandler,
    publicKey: payer.publicKey,
    editionMint: mint.publicKey,
    metadataData: metadata,
  });
  logDebug(createRes.txSummary.logMessages.join('\n'));

  assertTransactionSummary(t, createRes.txSummary, {
    fee: 5000,
    msgRx: [/Program.+metaq/i, /Instruction.+ Create Metadata Accounts/i],
  });

  logDebug('Mint Account: %s', mint.publicKey.toBase58());
  const mintAccount = await connection.getAccountInfo(mint.publicKey);
  logDebug({
    mintAccountOwner: mintAccount.owner.toBase58(),
    mintAccountDataBytes: mintAccount.data.byteLength,
  });

  t.ok(Edition.isCompatible(mintAccount.data), 'mint account data is mint edition');
  const editionData = EditionData.deserialize(mintAccount.data);
  dump(editionData);
  const edition: BN = editionData.edition;
  t.ok(edition.toNumber() > 0, 'greater zero edition number');
});
