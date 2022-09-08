import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';

const init = new InitTransactions();

killStuckProcess();

test('withdraw', async (t) => {
  const { fstTxHandler, payerPair, connection } = await init.payer();
  const items = 100;

  const data = {
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

  const { tx: transaction, candyMachine: address } = await init.create(
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

  const { tx: withdrawTransaction } = await init.withdraw(t, address, payerPair, fstTxHandler);
  await withdrawTransaction.assertSuccess(t);

  accountInfo = await connection.getAccountInfo(payerPair.publicKey);
  const updatedBalance = accountInfo.lamports;

  t.true(updatedBalance > balance, 'balance after withdraw must be greater');
});
