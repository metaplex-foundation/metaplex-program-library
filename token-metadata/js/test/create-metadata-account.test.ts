import test from 'tape';
import spok from 'spok';
import { Connection, Keypair } from '@solana/web3.js';
import {
  DataV2,
  Edition,
  EditionData,
  MetadataData,
  MetadataDataData,
  MetadataKey,
  TokenStandard,
} from '../src/mpl-token-metadata';
import { connectionURL, killStuckProcess } from './utils';
import {
  airdrop,
  assertTransactionSummary,
  PayerTransactionHandler,
} from '@metaplex-foundation/amman';
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import BN from 'bn.js';

import { logDebug } from './utils';
import { addLabel, isKeyOf } from './utils/address-labels';
import { createMetadata, createMetadataV2 } from './actions';

killStuckProcess();

const URI = 'uri';
const NAME = 'test';
const SYMBOL = 'sym';
const SELLER_FEE_BASIS_POINTS = 10;

test('create-metadata-account: success', async (t) => {
  const payer = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);
  const mint = await Token.createMint(
    connection,
    payer,
    payer.publicKey,
    null,
    6,
    TOKEN_PROGRAM_ID,
  );

  addLabel('create:mint', mint.publicKey);

  const initMetadataData = new MetadataDataData({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
  });

  const { createTxDetails, metadata } = await createMetadata({
    transactionHandler,
    publicKey: payer.publicKey,
    editionMint: mint.publicKey,
    metadataData: initMetadataData,
  });

  addLabel('create:metadata', metadata);
  logDebug(createTxDetails.txSummary.logMessages.join('\n'));

  assertTransactionSummary(t, createTxDetails.txSummary, {
    fee: 5000,
    msgRx: [/Program.+metaq/i, /Instruction.+ Create Metadata Accounts/i],
  });
  const metadataAccount = await connection.getAccountInfo(metadata);
  logDebug({
    metadataAccountOwner: metadataAccount.owner.toBase58(),
    metadataAccountDataBytes: metadataAccount.data.byteLength,
  });

  const metadataData = MetadataData.deserialize(<Buffer>metadataAccount.data);
  spok(t, metadataData, {
    $topic: 'metadataData',
    key: MetadataKey.MetadataV1,
    updateAuthority: isKeyOf(payer),
    mint: isKeyOf(mint.publicKey),
    data: {
      name: NAME,
      symbol: SYMBOL,
      uri: URI,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    },
    primarySaleHappened: 0,
    isMutable: 1,
  });

  const mintAccount = await connection.getAccountInfo(mint.publicKey);
  logDebug({
    mintAccountOwner: mintAccount.owner.toBase58(),
    mintAccountDataBytes: mintAccount.data.byteLength,
  });

  t.ok(Edition.isCompatible(mintAccount.data), 'mint account data is mint edition');

  const editionData = EditionData.deserialize(<Buffer>mintAccount.data);
  const edition: BN = editionData.edition;
  t.ok(edition.toNumber() > 0, 'greater zero edition number');
});

test('create-metadata-account-v2: success', async (t) => {
  const payer = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const mint = await Token.createMint(
    connection,
    payer,
    payer.publicKey,
    null,
    6,
    TOKEN_PROGRAM_ID,
  );
  addLabel('create:mint', mint.publicKey);

  const initMetadataData = new DataV2({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
    collection: null,
    uses: null,
  });
  const { createTxDetails, metadata } = await createMetadataV2({
    transactionHandler,
    publicKey: payer.publicKey,
    mint: mint.publicKey,
    metadataData: initMetadataData,
    updateAuthority: payer.publicKey,
  });

  addLabel('create:metadata', metadata);
  logDebug(createTxDetails.txSummary.logMessages.join('\n'));

  assertTransactionSummary(t, createTxDetails.txSummary, {
    fee: 5000,
    msgRx: [/Program.+metaq/i, /Instruction.+ Create Metadata Accounts/i],
  });
  const metadataAccount = await connection.getAccountInfo(metadata);
  logDebug({
    metadataAccountOwner: metadataAccount.owner.toBase58(),
    metadataAccountDataBytes: metadataAccount.data.byteLength,
  });

  const metadataData = MetadataData.deserialize(<Buffer>metadataAccount.data);
  spok(t, metadataData, {
    $topic: 'metadataData',
    key: MetadataKey.MetadataV1,
    updateAuthority: isKeyOf(payer),
    mint: isKeyOf(mint.publicKey),
    data: {
      name: NAME,
      symbol: SYMBOL,
      uri: URI,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    },
    primarySaleHappened: 0,
    isMutable: 1,
    tokenStandard: TokenStandard.FungibleAsset, // Since there is no master edition
  });
});
