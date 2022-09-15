import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup/';
import * as program from '../src/generated';
import { drain } from './utils/minter';
import spok from 'spok';

const API = new InitTransactions();

killStuckProcess();

test('mint (authority)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 10;

  const data: program.CandyMachineData = {
    itemsAvailable: items,
    symbol: 'CORE',
    sellerFeeBasisPoints: 500,
    maxSupply: 0,
    isMutable: true,
    creators: [
      {
        address: payerPair.publicKey,
        verified: false,
        percentageShare: 100,
      },
    ],
    configLineSettings: {
      prefixName: 'TEST ',
      nameLength: 10,
      prefixUri: 'https://arweave.net/',
      uriLength: 50,
      isSequential: false,
    },
    hiddenSettings: null,
  };

  const { tx: transaction, candyMachine: address } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  const lines: program.ConfigLine[] = [];

  for (let i = 0; i < items; i++) {
    lines[i] = {
      name: `NFT #${i + 1}`,
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }

  const { txs } = await API.addConfigLines(t, address, payerPair, lines);
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertSuccess(t);
  }

  const { tx: mintTransaction } = await API.mint(t, address, payerPair, fstTxHandler, connection);
  await mintTransaction.assertSuccess(t);
});

test('mint (sequential)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 10;

  const data: program.CandyMachineData = {
    itemsAvailable: items,
    symbol: 'CORE',
    sellerFeeBasisPoints: 500,
    maxSupply: 0,
    isMutable: true,
    creators: [
      {
        address: payerPair.publicKey,
        verified: false,
        percentageShare: 100,
      },
    ],
    configLineSettings: {
      prefixName: '$ID+1$',
      nameLength: 0,
      prefixUri: 'https://arweave.net/',
      uriLength: 50,
      isSequential: true,
    },
    hiddenSettings: null,
  };

  const { tx: transaction, candyMachine: address } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  const lines: program.ConfigLine[] = [];

  for (let i = 0; i < items; i++) {
    lines[i] = {
      name: '',
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }

  const { txs } = await API.addConfigLines(t, address, payerPair, lines);
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertSuccess(t);
  }

  // darining the candy machine
  const indices = await drain(t, address, payerPair, fstTxHandler, connection);
  const expected = Array.from({ length: indices.length }, (x, i) => i + 1);
  spok(t, indices, expected);

  // candy machine should be empty
  const { tx: mintTransaction } = await API.mint(t, address, payerPair, fstTxHandler, connection);
  await mintTransaction.assertError(t, /Candy machine is empty/i);
});

test('mint (random)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 10;

  const data: program.CandyMachineData = {
    itemsAvailable: items,
    symbol: 'CORE',
    sellerFeeBasisPoints: 500,
    maxSupply: 0,
    isMutable: true,
    creators: [
      {
        address: payerPair.publicKey,
        verified: false,
        percentageShare: 100,
      },
    ],
    configLineSettings: {
      prefixName: '$ID+1$',
      nameLength: 0,
      prefixUri: 'https://arweave.net/',
      uriLength: 50,
      isSequential: false,
    },
    hiddenSettings: null,
  };

  const { tx: transaction, candyMachine: address } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  const lines: program.ConfigLine[] = [];

  for (let i = 0; i < items; i++) {
    lines[i] = {
      name: '',
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }

  const { txs } = await API.addConfigLines(t, address, payerPair, lines);
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertSuccess(t);
  }

  // darining the candy machine
  const indices = await drain(t, address, payerPair, fstTxHandler, connection);
  const expected = Array.from({ length: indices.length }, (x, i) => i + 1);
  t.notDeepEqual(indices, expected);
  // sort the indices to test duplicates
  indices.sort(function (a, b) {
    return a - b;
  });
  spok(t, indices, expected);

  // candy machine should be empty
  const { tx: mintTransaction } = await API.mint(t, address, payerPair, fstTxHandler, connection);
  await mintTransaction.assertError(t, /Candy machine is empty/i);
});

test.only('mint (hidden settings)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 10;

  const data: program.CandyMachineData = {
    itemsAvailable: items,
    symbol: 'CORE',
    sellerFeeBasisPoints: 500,
    maxSupply: 0,
    isMutable: true,
    creators: [
      {
        address: payerPair.publicKey,
        verified: false,
        percentageShare: 100,
      },
    ],
    configLineSettings: null,
    hiddenSettings: {
      name: '$ID+1$',
      uri: 'https://arweave.net/uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
      hash: Buffer.from('74bac30d82a0baa41dd2bee4b41bbc36').toJSON().data,
    },
  };

  const { tx: transaction, candyMachine: address } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  // darining the candy machine
  const indices = await drain(t, address, payerPair, fstTxHandler, connection);
  const expected = Array.from({ length: indices.length }, (x, i) => i + 1);
  spok(t, indices, expected);

  // candy machine should be empty
  const { tx: mintTransaction } = await API.mint(t, address, payerPair, fstTxHandler, connection);
  await mintTransaction.assertError(t, /Candy machine is empty/i);
});
