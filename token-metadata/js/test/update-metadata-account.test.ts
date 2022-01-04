import test from 'tape';

import { Data, UpdateMetadata } from '../src/mpl-token-metadata';
import {
  killStuckProcess,
  initMetadata,
  getMetadata,
  assertMetadataUnchanged,
  assertDataUnchanged,
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

  const updatedMetadata = await getMetadata(connection, metadata);
  t.ok(updatedMetadata.primarySaleHappened, 'after update sale happened');
  assertMetadataUnchanged(t, initialMetadata, updatedMetadata, 'primarySaleHappened');
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

  const updatedMetadata = await getMetadata(connection, metadata);
  assertMetadataUnchanged(t, initialMetadata, updatedMetadata);
});

test('update-metadata-account: uri', async (t) => {
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await initMetadata();

  const tx = new UpdateMetadata(
    {},
    {
      metadata,
      metadataData: new Data({ ...initialMetadata.data, uri: `${URI}-updated` }),
      updateAuthority: payer.publicKey,
    },
  );
  await transactionHandler.sendAndConfirmTransaction(tx, [payer]);

  const updatedMetadata = await getMetadata(connection, metadata);
  t.equal(updatedMetadata.data.uri, `${URI}-updated`, 'updates uri');
  assertDataUnchanged(t, initialMetadata.data, updatedMetadata.data, ['uri']);
});

test('update-metadata-account: name and symbol', async (t) => {
  const { connection, transactionHandler, payer, metadata, initialMetadata } = await initMetadata();

  const tx = new UpdateMetadata(
    {},
    {
      metadata,
      metadataData: new Data({
        ...initialMetadata.data,
        name: `${NAME}-updated`,
        symbol: `${SYMBOL}++`,
      }),
      updateAuthority: payer.publicKey,
    },
  );
  await transactionHandler.sendAndConfirmTransaction(tx, [payer]);

  const updatedMetadata = await getMetadata(connection, metadata);
  t.equal(updatedMetadata.data.name, `${NAME}-updated`, 'updates name');
  t.equal(updatedMetadata.data.symbol, `${SYMBOL}++`, 'updates symbol');
  assertDataUnchanged(t, initialMetadata.data, updatedMetadata.data, [
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
      metadataData: new Data({
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
