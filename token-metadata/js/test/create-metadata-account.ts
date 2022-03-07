import test from 'tape';
import { Connection, Transaction } from '@solana/web3.js';
import { assertHasError, assertMetadataAccount, killStuckProcess } from './utils';
import {
  CannotVerifyAnotherCreatorError,
  CreatorsMustBeAtleastOneError,
  CreatorsTooLongError,
  DuplicateCreatorAddressError,
  InvalidUseMethodError,
  NumericalOverflowErrorError,
  ShareTotalMustBe100Error,
  UseMethod,
  Uses,
} from '../src/generated';
import {
  assertConfirmedTransaction,
  assertTransactionSummary,
  ConfirmedTransactionDetails,
  LOCALHOST,
} from '@metaplex-foundation/amman';

import { amman } from './utils';
import {
  createMetadataAccount,
  CreateMetadataAccountSetup,
  Creator,
  DataV2,
  Metadata,
} from '../src/mpl-token-metadata';

killStuckProcess();

const URI = 'uri';
const NAME = 'test';
const SYMBOL = 'sym';
const SELLER_FEE_BASIS_POINTS = 10;
const CREATOR = '👩‍🎨';

const SUCCESS_RXS = [/InitializeMint/i, /Create Metadata Accounts v2/i, /success/i];
const DATA_V2: DataV2 = {
  uri: URI,
  name: NAME,
  symbol: SYMBOL,
  sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
  creators: null,
  collection: null,
  uses: null,
};

// -----------------
// Plain
// -----------------
test('create-metadata-account: non-mutable without optional params', async (t) => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const [payer, payerPair] = amman.genKeypair('payer');
  const transactionHandler = amman.payerTransactionHandler(connection, payerPair);
  await amman.airdrop(connection, payer, 1);

  const setup = await CreateMetadataAccountSetup.create(connection, {
    payer: payer,
  }).createMintAccount();

  amman.addr.addLabels(setup);

  const createMetadataAccountIx = await createMetadataAccount(setup, DATA_V2, false);

  const tx = new Transaction().add(...setup.instructions).add(createMetadataAccountIx);
  const res = await transactionHandler.sendAndConfirmTransaction(
    tx,
    setup.signers,
    { skipPreflight: true },
    'Create Mint + Metadata',
  );
  assertConfirmedTransaction(t, res.txConfirmed);
  assertTransactionSummary(t, res.txSummary, {
    msgRx: SUCCESS_RXS,
  });
  const metadataAccount = await Metadata.fromAccountAddress(connection, setup.metadata);
  assertMetadataAccount(t, metadataAccount, setup, DATA_V2, {
    isMutable: false,
    primarySaleHappened: false,
  });
});

// -----------------
// Creators
// -----------------
test('create-metadata-account:with creators, Failure Cases', async (t) => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const [payer, payerPair] = amman.genKeypair('payer');
  const transactionHandler = amman.payerTransactionHandler(connection, payerPair);
  await amman.airdrop(connection, payer, 1);

  const setup = await CreateMetadataAccountSetup.create(connection, {
    payer: payer,
  }).createMintAccount();
  amman.addr.addLabels(setup);

  async function exec(creators: Creator[], label: string) {
    const data = { ...DATA_V2, creators };
    const createMetadataAccountIx = await createMetadataAccount(setup, data, false);

    const tx = new Transaction().add(...setup.instructions).add(createMetadataAccountIx);
    const res = await transactionHandler.sendAndConfirmTransaction(
      tx,
      setup.signers,
      { skipPreflight: true },
      `🌱 Metata with ${label}`,
    );
    return res;
  }

  const [creator1] = amman.genKeypair('creator 1');
  const [creator2] = amman.genKeypair('creator 2');
  const [creator3] = amman.genKeypair('creator 3');
  const [creator4] = amman.genKeypair('creator 4');
  const [creator5] = amman.genKeypair('creator 5');
  {
    const label = 'six unverified ';
    t.comment(`++++ ${label}`);
    const res = await exec(
      [
        { address: payer, share: 1, verified: false },
        { address: creator1, share: 1, verified: false },
        { address: creator2, share: 1, verified: false },
        { address: creator3, share: 1, verified: false },
        { address: creator4, share: 1, verified: false },
        { address: creator5, share: 95, verified: false },
      ],
      label,
    );
    assertHasError(t, res, CreatorsTooLongError);
  }
  {
    const label = `four unverified ${CREATOR}s with one duplicate`;
    t.comment(`++++ ${label}`);
    const res = await exec(
      [
        { address: payer, share: 25, verified: false },
        { address: creator1, share: 25, verified: false },
        { address: creator2, share: 25, verified: false },
        { address: creator1, share: 25, verified: false },
      ],
      label,
    );
    assertHasError(t, res, DuplicateCreatorAddressError);
  }
  {
    const label = `empty ${CREATOR}s`;
    t.comment(`++++ ${label}`);
    const res = await exec([], label);
    assertHasError(t, res, CreatorsMustBeAtleastOneError);
  }
  {
    const label = `three unverified ${CREATOR}s 3 total shares`;
    t.comment(`++++ ${label}`);
    const res = await exec(
      [
        { address: creator1, share: 1, verified: false },
        { address: payer, share: 1, verified: false },
        { address: creator2, share: 1, verified: false },
      ],
      label,
    );
    assertHasError(t, res, ShareTotalMustBe100Error);
  }
  {
    const label = `three unverified ${CREATOR}s 300 total shares`;
    t.comment(`++++ ${label}`);
    const res = await exec(
      [
        { address: creator1, share: 100, verified: false },
        { address: payer, share: 100, verified: false },
        { address: creator2, share: 100, verified: false },
      ],
      label,
    );
    assertHasError(t, res, NumericalOverflowErrorError);
  }
  {
    const label = `three unverified ${CREATOR}s 101 total shares`;
    t.comment(`++++ ${label}`);
    const res = await exec(
      [
        { address: creator1, share: 90, verified: false },
        { address: payer, share: 1, verified: false },
        { address: creator2, share: 10, verified: false },
      ],
      label,
    );
    assertHasError(t, res, ShareTotalMustBe100Error);
  }
  {
    const label = `three ${CREATOR}s non-payer verified`;
    t.comment(`++++ ${label}`);
    const res = await exec(
      [
        { address: creator1, share: 90, verified: true },
        { address: payer, share: 1, verified: false },
        { address: creator2, share: 9, verified: false },
      ],
      label,
    );
    assertHasError(t, res, CannotVerifyAnotherCreatorError);
  }
});

test('create-metadata-account: with creators, Success Cases', async (t) => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const [payer, payerPair] = amman.genKeypair('payer');
  const transactionHandler = amman.payerTransactionHandler(connection, payerPair);
  await amman.airdrop(connection, payer, 1);

  async function check(
    creators: Creator[],
    label: string,
  ): Promise<[ConfirmedTransactionDetails, CreateMetadataAccountSetup, DataV2]> {
    const setup = await CreateMetadataAccountSetup.create(connection, {
      payer: payer,
    }).createMintAccount();
    amman.addr.addLabels(setup);

    const data = { ...DATA_V2, creators };
    const createMetadataAccountIx = await createMetadataAccount(setup, data, false);

    const tx = new Transaction().add(...setup.instructions).add(createMetadataAccountIx);
    const res = await transactionHandler.sendAndConfirmTransaction(
      tx,
      setup.signers,
      { skipPreflight: true },
      `🌱 Metata with ${label}`,
    );
    assertConfirmedTransaction(t, res.txConfirmed);
    assertTransactionSummary(t, res.txSummary, {
      msgRx: SUCCESS_RXS,
    });
    const metadataAccount = await Metadata.fromAccountAddress(connection, setup.metadata);
    assertMetadataAccount(t, metadataAccount, setup, data);

    return [res, setup, data];
  }

  const [creator1] = amman.genKeypair('creator 1');
  const [creator2] = amman.genKeypair('creator 2');
  {
    const label = `three unverified ${CREATOR}s 100 total shares`;
    t.comment(`++++ ${label}`);
    await check(
      [
        { address: creator1, share: 90, verified: false },
        { address: payer, share: 1, verified: false },
        { address: creator2, share: 9, verified: false },
      ],
      label,
    );
  }
  {
    const label = `three ${CREATOR}s payer verified`;
    t.comment(`++++ ${label}`);
    await check(
      [
        { address: creator1, share: 90, verified: false },
        { address: payer, share: 1, verified: true },
        { address: creator2, share: 9, verified: false },
      ],
      label,
    );
  }
});

// -----------------
// Uses
// -----------------
test('create-metadata-account: with uses, Success Cases', async (t) => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const [payer, payerPair] = amman.genKeypair('payer');
  const transactionHandler = amman.payerTransactionHandler(connection, payerPair);
  await amman.airdrop(connection, payer, 1);

  async function check(
    uses: Uses,
    label: string,
  ): Promise<[ConfirmedTransactionDetails, CreateMetadataAccountSetup, DataV2]> {
    const setup = await CreateMetadataAccountSetup.create(connection, {
      payer: payer,
    }).createMintAccount();
    amman.addr.addLabels(setup);

    const data = { ...DATA_V2, uses };
    const createMetadataAccountIx = await createMetadataAccount(setup, data, false);

    const tx = new Transaction().add(...setup.instructions).add(createMetadataAccountIx);
    const res = await transactionHandler.sendAndConfirmTransaction(
      tx,
      setup.signers,
      { skipPreflight: true },
      `🌱 Metata with ${label}`,
    );
    assertConfirmedTransaction(t, res.txConfirmed);
    assertTransactionSummary(t, res.txSummary, {
      msgRx: SUCCESS_RXS,
    });
    const metadataAccount = await Metadata.fromAccountAddress(connection, setup.metadata);
    assertMetadataAccount(t, metadataAccount, setup, data);
    return [res, setup, data];
  }

  {
    const label = 'uses mulitple 5/10';
    t.comment(`++++ ${label}`);
    await check({ remaining: 5, total: 10, useMethod: UseMethod.Multiple }, label);
  }
  {
    const label = 'uses mulitple 10/5';
    t.comment(`++++ ${label}`);
    // TODO(thlorenz): This should fail!!!
    await check({ remaining: 10, total: 5, useMethod: UseMethod.Multiple }, label);
  }
  {
    const label = 'uses single 1/1';
    t.comment(`++++ ${label}`);
    await check({ remaining: 1, total: 1, useMethod: UseMethod.Single }, label);
  }
  {
    const label = 'uses burn 0/0';
    t.comment(`++++ ${label}`);
    await check({ remaining: 0, total: 0, useMethod: UseMethod.Burn }, label);
  }
  {
    const label = 'uses burn 5/10';
    t.comment(`++++ ${label}`);
    await check({ remaining: 5, total: 10, useMethod: UseMethod.Burn }, label);
  }
});

test('create-metadata-account: with uses, Failure Cases', async (t) => {
  const connection = new Connection(LOCALHOST, 'confirmed');
  const [payer, payerPair] = amman.genKeypair('payer');
  const transactionHandler = amman.payerTransactionHandler(connection, payerPair);
  await amman.airdrop(connection, payer, 1);

  // eslint-disable-next-line @typescript-eslint/ban-types
  async function check(uses: Uses, expectedError: Function, label: string) {
    const setup = await CreateMetadataAccountSetup.create(connection, {
      payer: payer,
    }).createMintAccount();
    amman.addr.addLabels(setup);

    const data = { ...DATA_V2, uses };
    const createMetadataAccountIx = await createMetadataAccount(setup, data, false);

    const tx = new Transaction().add(...setup.instructions).add(createMetadataAccountIx);
    const res = await transactionHandler.sendAndConfirmTransaction(
      tx,
      setup.signers,
      { skipPreflight: true },
      `🌱 Metata with ${label}`,
    );
    assertHasError(t, res, expectedError);
  }
  {
    const label = 'uses mulitple 0/0';
    t.comment(`++++ ${label}`);
    await check(
      { remaining: 0, total: 0, useMethod: UseMethod.Multiple },
      InvalidUseMethodError,
      label,
    );
  }
  {
    const label = 'uses single 0/1';
    t.comment(`++++ ${label}`);
    await check(
      { remaining: 0, total: 1, useMethod: UseMethod.Single },
      InvalidUseMethodError,
      label,
    );
  }
  {
    const label = 'uses single 1/0';
    t.comment(`++++ ${label}`);
    await check(
      { remaining: 1, total: 0, useMethod: UseMethod.Single },
      InvalidUseMethodError,
      label,
    );
  }
  {
    const label = 'uses single 1/2';
    t.comment(`++++ ${label}`);
    await check(
      { remaining: 1, total: 2, useMethod: UseMethod.Single },
      InvalidUseMethodError,
      label,
    );
  }
});
