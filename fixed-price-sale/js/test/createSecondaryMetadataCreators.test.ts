import test from 'tape';
import {
  assertConfirmedTransaction,
  assertError,
  defaultSendOptions,
} from '@metaplex-foundation/amman';

import { CreatorAccountData } from '../src';
import { killStuckProcess, logDebug } from './utils';
import { createPrerequisites, createStore, initSellingResource } from './actions';
import { createSecondaryMetadataCreators } from './transactions';

killStuckProcess();

test('create-secondary-metadata-creators: success', async (t) => {
  const { payer, connection, transactionHandler } = await createPrerequisites();

  const store = await createStore({
    test: t,
    transactionHandler,
    payer,
    connection,
    params: {
      name: 'Store',
      description: 'Description',
    },
  });

  const { metadata } = await initSellingResource({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    maxSupply: 100,
  });

  const creator = CreatorAccountData.fromArgs({
    address: payer.publicKey,
    share: 100,
    verified: false,
  });

  const { secondaryMetadataCreators, createSecondaryMetadataCreatorsTx } =
    await createSecondaryMetadataCreators({
      test: t,
      transactionHandler,
      payer,
      connection,
      metadata,
      creators: [creator],
    });

  const createSecondaryMetadataCreatorsRes = await transactionHandler.sendAndConfirmTransaction(
    createSecondaryMetadataCreatorsTx,
    [payer],
    defaultSendOptions,
  );

  logDebug(`secondary-metadata-creators: ${secondaryMetadataCreators.toBase58()}`);
  assertConfirmedTransaction(t, createSecondaryMetadataCreatorsRes.txConfirmed);
});

test('create-secondary-metadata-creators: empty creators', async (t) => {
  const { payer, connection, transactionHandler } = await createPrerequisites();

  const store = await createStore({
    test: t,
    transactionHandler,
    payer,
    connection,
    params: {
      name: 'Store',
      description: 'Description',
    },
  });

  const { metadata } = await initSellingResource({
    test: t,
    transactionHandler,
    payer,
    connection,
    store: store.publicKey,
    maxSupply: 100,
  });

  const creators: CreatorAccountData[] = [];

  const { createSecondaryMetadataCreatorsTx } = await createSecondaryMetadataCreators({
    test: t,
    transactionHandler,
    payer,
    connection,
    metadata,
    creators,
  });

  try {
    await transactionHandler.sendAndConfirmTransaction(
      createSecondaryMetadataCreatorsTx,
      [payer],
      defaultSendOptions,
    );
    t.fail('expected transaction to fail');
  } catch (err) {
    assertError(t, err, [/custom program error/i, /0x1791/i]);
  }
});
