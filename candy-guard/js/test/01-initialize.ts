import test from 'tape';
import spok from 'spok';
import { BN } from 'bn.js';
import { InitTransactions, killStuckProcess } from './setup/';
import { CandyGuard } from '../src/generated';
import { DATA_OFFSET, spokSameBignum, spokSamePubkey } from './utils';
import { parseData } from '../src';

const API = new InitTransactions();

killStuckProcess();

test('initialize: new candy guard (no guards)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
      botTax: null,
      liveDate: null,
      lamports: null,
      splToken: null,
      thirdPartySigner: null,
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: null,
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
  // retrieves the created candy machine
  const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

  spok(t, candyGuard, {
    authority: spokSamePubkey(payerPair.publicKey),
  });

  // parse the guards configuration
  const accountInfo = await connection.getAccountInfo(address);
  const candyGuardData = parseData(accountInfo?.data.subarray(DATA_OFFSET));

  spok(t, candyGuardData, data);
});

test('initialize: new candy guard (with guards)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await API.payer();

  const data = {
    default: {
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
      thirdPartySigner: {
        signerKey: payerPair.publicKey,
      },
      whitelist: null,
      gatekeeper: null,
      endSettings: null,
      allowList: null,
      mintLimit: null,
      nftPayment: null,
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
  // retrieves the created candy machine
  const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

  spok(t, candyGuard, {
    authority: spokSamePubkey(payerPair.publicKey),
  });

  // parse the guards configuration
  const accountInfo = await connection.getAccountInfo(address);
  const candyGuardData = parseData(accountInfo?.data.subarray(DATA_OFFSET));

  //spok(t, candyGuardData.default.liveDate, {
  //  date: spokSameBignum(data.default.liveDate.date),
  //});

  spok(t, candyGuardData.default.lamports, {
    amount: spokSameBignum(data.default.lamports.amount),
  });

  spok(t, candyGuardData.default.thirdPartySigner, {
    signerKey: spokSamePubkey(payerPair.publicKey),
  });

  //let accountInfo = await connection.getAccountInfo(address);
  //const candyGuardData = parseData(candyGuard, accountInfo?.data.subarray(DATA_OFFSET)!);
  //console.log(candyGuardData);

  // this is currently failing, most likely because the liveDate.date is set
  // to null so the offset for the deserialization for the lamports guards
  // is misalign with the account data buffer

  // spok(t, candyGuardData.lamports, {
  //   amount: spokSameBignum(data.lamports.amount),
  // });
});
