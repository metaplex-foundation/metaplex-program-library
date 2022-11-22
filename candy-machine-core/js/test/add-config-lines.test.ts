import {
  AddConfigLinesInstructionAccounts,
  AddConfigLinesInstructionArgs,
  CandyMachineData,
  ConfigLine,
  createAddConfigLinesInstruction,
} from '../src/generated';
import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { Transaction } from '@solana/web3.js';

killStuckProcess();

test('add_config_lines', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 100;

  const data: CandyMachineData = {
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

  const lines: ConfigLine[] = [];

  for (let i = 0; i < items; i++) {
    lines[i] = {
      name: `NFT #${i + 1}`,
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }
  const { txs } = await API.addConfigLines(t, address, payerPair, lines, 0);
  // confirms that all lines have been written
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertSuccess(t, [/New config line added/i]);
  }
});

test('add_config_lines (hidden settings)', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 10;

  const data: CandyMachineData = {
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
      name: 'Hidden NFT',
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

  const lines: ConfigLine[] = [];

  for (let i = 0; i < items; i++) {
    lines[i] = {
      name: `NFT #${i + 1}`,
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }
  const { txs } = await API.addConfigLines(t, address, payerPair, lines, 0);
  // this should fail since hiddenSettings do not have config lines
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertError(t, /do not have config lines/i);
  }
});

test('add_config_lines (incomplete)', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 10;

  const data: CandyMachineData = {
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

  const lines: ConfigLine[] = [];

  for (let i = 0; i < items - 5; i++) {
    lines[i] = {
      name: `NFT #${i + 1}`,
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }
  const { txs } = await API.addConfigLines(t, address, payerPair, lines, 0);
  // confirms that all lines have been written
  for (const tx of txs) {
    await fstTxHandler
      .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
      .assertSuccess(t, [/New config line added/i]);
  }

  const { tx: mintTransaction } = await API.mint(t, address, payerPair, fstTxHandler, connection);
  await mintTransaction.assertError(t, /Not all config lines were added/i);
});

test('add_config_lines: invalid start index', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 100;

  const data: CandyMachineData = {
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

  const lines: ConfigLine[] = [];

  for (let i = 0; i < items; i++) {
    lines[i] = {
      name: `NFT #${i + 1}`,
      uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
    };
  }
  const { txs } = await API.addConfigLines(t, address, payerPair, lines, 100);

  await fstTxHandler
    .sendAndConfirmTransaction(txs[0], [payerPair], 'tx: AddConfigLines')
    .assertError(t, /Index greater than length/i);
});

test('add_config_lines: empty config lines', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler, payerPair, connection } = await API.payer();
  const items = 100;

  const data: CandyMachineData = {
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

  const accounts: AddConfigLinesInstructionAccounts = {
    candyMachine: address,
    authority: payerPair.publicKey,
  };

  const args: AddConfigLinesInstructionArgs = {
    configLines: [],
    index: 0,
  };

  const ix = createAddConfigLinesInstruction(accounts, args);
  const tx = await new Transaction().add(ix);

  await fstTxHandler
    .sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines')
    .assertSuccess(t, [/Config lines array empty/i]);
});
