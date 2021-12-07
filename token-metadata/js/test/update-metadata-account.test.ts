import test from 'tape';
import spok from 'spok';

import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { MetadataData, MetadataDataData, UpdateMetadata } from '../src/mpl-token-metadata';
import {
  connectionURL,
  airdrop,
  PayerTransactionHandler,
  killStuckProcess,
  assertError,
} from './utils';

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
  except?: keyof MetadataData,
) {
  const x = { ...initial };
  if (except != null) {
    delete x[except];
  }
  delete x.data.creators;

  const y = { $topic: `no change except '${except}' on metadata`, ...updated };
  if (except != null) {
    delete y[except];
  }
  delete y.data.creators;

  spok(t, x, y);
}

async function assertMetadataDataDataUnchanged(
  t: test.Test,
  initial: MetadataDataData,
  updated: MetadataDataData,
  except: (keyof MetadataDataData)[],
) {
  const x = { ...initial };
  except.forEach((f) => delete x[f]);
  delete x.creators;

  const y = { $topic: `no change except '${except}' on metadataData`, ...updated };
  except.forEach((f) => delete y[f]);
  delete y.creators;

  spok(t, x, y);
}

// -----------------
// Success Cases
// -----------------
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

test('update-metadata-account: update with same data', async (t) => {
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
  assertMetadataDataUnchanged(t, initialMetadata, updatedMetadata);
});

test('update-metadata-account: uri', async (t) => {
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await init();

  const tx = new UpdateMetadata(
    {},
    {
      metadata,
      metadataData: new MetadataDataData({ ...initialMetadata.data, uri: `${URI}-updated` }),
      updateAuthority: payer.publicKey,
    },
  );
  await transactionHandler.sendAndConfirmTransaction(tx, [payer]);

  const updatedMetadata = await getMetadataData(connection, metadata);
  t.equal(updatedMetadata.data.uri, `${URI}-updated`, 'updates uri');
  assertMetadataDataDataUnchanged(t, initialMetadata.data, updatedMetadata.data, ['uri']);
});

test('update-metadata-account: name and symbol', async (t) => {
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await init();

  const tx = new UpdateMetadata(
    {},
    {
      metadata,
      metadataData: new MetadataDataData({
        ...initialMetadata.data,
        name: `${NAME}-updated`,
        symbol: `${SYMBOL}++`,
      }),
      updateAuthority: payer.publicKey,
    },
  );
  await transactionHandler.sendAndConfirmTransaction(tx, [payer]);

  const updatedMetadata = await getMetadataData(connection, metadata);
  t.equal(updatedMetadata.data.name, `${NAME}-updated`, 'updates name');
  t.equal(updatedMetadata.data.symbol, `${SYMBOL}++`, 'updates symbol');
  assertMetadataDataDataUnchanged(t, initialMetadata.data, updatedMetadata.data, [
    'name',
    'symbol',
  ]);
});

// -----------------
// Failure Cases
// -----------------

// TODO: at this point lots of success cases are tested, however tests for
// incorrect inputs, etc. should be added ASAP like the below example
test('update-metadata-account: update symbol too long', async (t) => {
  const { transactionHandler, payer, metadata, initialMetadata } = await init();

  const tx = new UpdateMetadata(
    {},
    {
      metadata,
      metadataData: new MetadataDataData({
        ...initialMetadata.data,
        symbol: `${SYMBOL}-too-long`,
      }),
      updateAuthority: payer.publicKey,
    },
  );
  try {
    await transactionHandler.sendAndConfirmTransaction(tx, [payer]);
    t.fail('expected transaction to fail');
  } catch (err) {
    assertError(t, err, [/custom program error/i, /Symbol too long/i]);
  }
});
