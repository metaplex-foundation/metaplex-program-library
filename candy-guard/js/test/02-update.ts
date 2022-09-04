import test from 'tape';
import spok from 'spok';
import { InitTransactions, killStuckProcess } from './setup';
import { CandyGuard } from '../src/generated';
import { spokSameBignum, spokSamePubkey } from './utils';
import { BN } from 'bn.js';

const API = new InitTransactions();

killStuckProcess();

test('update: enable guards', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
      uuid: [1, 2, 3, 4, 5, 6],
      botTax: null,
      liveDate: null,
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
    },
    groups: null,
  };

  const { tx: transaction, candyGuard: address } = await API.initialize(
    t,
    data,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  let accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const balance = accountInfo?.lamports!;

  const updateData = {
    default: {
      uuid: [1, 2, 3, 4, 5, 6],
      botTax: {
        lamports: new BN(100000000),
        lastInstruction: true,
      },
      liveDate: {
        date: null,
      },
      lamports: {
        amount: new BN(100000000),
      },
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
    },
    groups: null
  };

  const { tx: updateTransaction } = await API.update(
    t,
    address,
    updateData,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await updateTransaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

  spok(t, candyGuard, {
    authority: spokSamePubkey(payerPair.publicKey),
  });

  accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const updatedBalance = accountInfo?.lamports!;

  t.true(updatedBalance < balance, 'balance after update must be lower');
});

test('update: disable guards', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
      uuid: [0, 0, 0, 0, 0, 1],
      botTax: {
        lamports: new BN(100000000),
        lastInstruction: true,
      },
      liveDate: {
        date: null,
      },
      lamports: {
        amount: new BN(100000000),
      },
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
    },
    groups: [{
      uuid: [0, 0, 0, 0, 0, 2],
      botTax: null,
      liveDate: null,
      lamports: {
        amount: new BN(500),
      },
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
    }]
  };

  const { tx: transaction, candyGuard: address } = await API.initialize(
    t,
    data,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await transaction.assertSuccess(t);

  let accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const balance = accountInfo?.lamports!;

  const updateData = {
    default: {
      uuid: [1, 2, 3, 4, 5, 6],
      botTax: null,
      liveDate: null,
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null
    },
    groups: null
  };

  const { tx: updateTransaction } = await API.update(
    t,
    address,
    updateData,
    payerPair,
    fstTxHandler,
  );
  // executes the transaction
  await updateTransaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

  spok(t, candyGuard, {
    authority: spokSamePubkey(payerPair.publicKey),
  });

  accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const updatedBalance = accountInfo?.lamports!;

  t.true(updatedBalance > balance, 'balance after update must be greater');
});
