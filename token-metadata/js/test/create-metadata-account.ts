import test from 'tape';
import { Connection, Transaction } from '@solana/web3.js';
import { assertMetadataAccount, killStuckProcess } from './utils';
import {
  assertConfirmedTransaction,
  assertTransactionSummary,
  LOCALHOST,
} from '@metaplex-foundation/amman';

import { amman } from './utils';
import {
  createMetadataAccount,
  CreateMetadataAccountSetup,
  DataV2,
  Metadata,
} from '../src/mpl-token-metadata';

killStuckProcess();

const URI = 'uri';
const NAME = 'test';
const SYMBOL = 'sym';
const SELLER_FEE_BASIS_POINTS = 10;

const DATA_V2: DataV2 = {
  uri: URI,
  name: NAME,
  symbol: SYMBOL,
  sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
  creators: null,
  collection: null,
  uses: null,
};

test('create-metadata-account: non-mutable without optional params', async (t) => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const [payer, payerPair] = amman.genKeypair('payer');
  const transactionHandler = amman.payerTransactionHandler(connection, payerPair);
  await amman.airdrop(connection, payer, 1);

  const setup = CreateMetadataAccountSetup.create(connection, { payer: payer });
  const mint = await setup.createMintAccount();
  amman.addr.addLabels({ mint });

  const createMetadataAccountIx = await createMetadataAccount(setup.asCompleted(), DATA_V2, false);

  const tx = new Transaction().add(...setup.instructions).add(createMetadataAccountIx);
  const res = await transactionHandler.sendAndConfirmTransaction(
    tx,
    setup.signers,
    { skipPreflight: true },
    'Create Mint + Metadata',
  );
  assertConfirmedTransaction(t, res.txConfirmed);
  assertTransactionSummary(t, res.txSummary, {
    msgRx: [/InitializeMint/i, /Create Metadata Accounts v2/i, /success/i],
  });
  const metadataAccount = await Metadata.fromAccountAddress(connection, setup.metadata);
  assertMetadataAccount(t, metadataAccount, setup, DATA_V2, {
    isMutable: false,
    primarySaleHappened: false,
  });
});
