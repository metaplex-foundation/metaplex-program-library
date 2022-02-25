import test from 'tape';

import { MetadataDataData, UpdateMetadata } from '../src/mpl-token-metadata';
import {
  killStuckProcess,
  initMetadata,
  getMetadataData,
  assertMetadataDataUnchanged,
  assertMetadataDataDataUnchanged,
  URI,
  NAME,
  SYMBOL,
} from './utils';
import { assertError } from '@metaplex-foundation/amman';

killStuckProcess();

// -----------------
// Success Cases
// -----------------

// TODO: are we supposed to be able to toggle this via an update transaction
// instead of using updatePrimarySaleHappend?
test('update-metadata-account: toggle primarySaleHappened', async (t) => {
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await initMetadata();

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
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await initMetadata();

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
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await initMetadata();

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
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await initMetadata();

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
  const { transactionHandler, payer, metadata, initialMetadata } = await initMetadata();

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
