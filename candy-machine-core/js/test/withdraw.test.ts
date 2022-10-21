import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { CandyMachineData } from '../src/generated';

killStuckProcess();

test('withdraw', async (t) => {
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

  let accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const balance = accountInfo.lamports;

  const { tx: withdrawTransaction } = await API.withdraw(t, address, payerPair, fstTxHandler);
  await withdrawTransaction.assertSuccess(t);

  accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const updatedBalance = accountInfo.lamports;

  t.true(updatedBalance > balance, 'balance after withdraw must be greater');
});
