import test from 'tape';
import spok from 'spok';

import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { MetadataData, MetadataDataData, UpdateMetadata } from '../src/mpl-token-metadata';
import { connectionURL, airdrop, PayerTransactionHandler, killStuckProcess } from './utils';

import { addLabel } from './utils/address-labels';
import { mintAndCreateMetadata } from './actions';

killStuckProcess();

const URI = 'uri';
const NAME = 'test';
const SYMBOL = 'sym';
const SELLER_FEE_BASIS_POINTS = 10;

async function init() {
  const payer = Keypair.generate();
  addLabel('payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const initMetadataData = new MetadataDataData({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
  });

  const { mint, metadata } = await mintAndCreateMetadata(
    connection,
    transactionHandler,
    payer,
    initMetadataData,
  );
  const initialMetadata = await getMetadataData(connection, metadata);
  return { connection, transactionHandler, payer, mint, metadata, initialMetadata };
}

async function getMetadataData(connection: Connection, metadata: PublicKey): Promise<MetadataData> {
  const metadataAccount = await connection.getAccountInfo(metadata);
  return MetadataData.deserialize(metadataAccount.data);
}

async function assertMetadataDataUnchanged(
  t: test.Test,
  initial: MetadataData,
  updated: MetadataData,
  except: keyof MetadataData,
) {
  const x = { ...initial };
  delete x[except];
  delete x.data.creators;

  const y = { $topic: `no change except '${except}' on metadataData`, ...updated };
  delete y[except];
  delete y.data.creators;

  spok(t, x, y);
}

// TODO: at this point only success cases are tested, however tests for
// incorrect inputs, etc. should be added ASAP
test('update-metadata-account: toggle primarySaleHappened', async (t) => {
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await init();

  t.notOk(initialMetadata.primarySaleHappened, 'initially sale has not happened');
  const tx = new UpdateMetadata(
    {},
    {
      metadata,
      updateAuthority: payer.publicKey,
      primarySaleHappened: true,
    },
  );
  await transactionHandler.sendAndConfirmTransaction(tx, [payer]);

  const updatedMetadata = await getMetadataData(connection, metadata);
  t.ok(updatedMetadata.primarySaleHappened, 'after update sale happened');
  assertMetadataDataUnchanged(t, initialMetadata, updatedMetadata, 'primarySaleHappened');
});

// TODO: this fails even though we pass the previously initialized data due
// to serialization problems
// `Error: Class MetadataDataData4 is missing in schema: data.data`
test.skip('update-metadata-account: update with same data', async (t) => {
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await init();

  const tx = new UpdateMetadata(
    {},
    {
      metadata,
      metadataData: initialMetadata.data,
      updateAuthority: payer.publicKey,
    },
  );
  await transactionHandler.sendAndConfirmTransaction(tx, [payer]);

  const updatedMetadata = await getMetadataData(connection, metadata);
  t.ok(updatedMetadata.primarySaleHappened, 'after update sale happened');
  assertMetadataDataUnchanged(t, initialMetadata, updatedMetadata, 'primarySaleHappened');
});
