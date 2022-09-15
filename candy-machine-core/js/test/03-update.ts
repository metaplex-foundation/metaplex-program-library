import test from 'tape';
import spok from 'spok';
import { CandyMachine } from 'src/generated';
import { InitTransactions, killStuckProcess } from './setup/';
import { spokSameBignum } from './utils';
import * as program from '../src/generated';

const init = new InitTransactions();

killStuckProcess();

test('update', async (t) => {
  const { fstTxHandler, payerPair, connection } = await init.payer();
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

  const { tx: transaction, candyMachine: address } = await init.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyMachine = await CandyMachine.fromAccountAddress(connection, address);

  spok(t, candyMachine.data, {
    sellerFeeBasisPoints: 500,
    isMutable: true,
    configLineSettings: {
      prefixName: 'TEST ',
      nameLength: 10,
      prefixUri: 'https://arweave.net/',
      uriLength: 50,
    },
  });

  data.sellerFeeBasisPoints = 1000;
  data.isMutable = false;
  data.configLineSettings.nameLength = 5;
  data.configLineSettings.uriLength = 25;

  const { tx: updateTransaction1 } = await init.updateCandyMachine(
    t,
    address,
    payerPair,
    data,
    fstTxHandler,
  );
  await updateTransaction1.assertSuccess(t);
  const updatedCandyMachine = await CandyMachine.fromAccountAddress(connection, address);

  spok(t, updatedCandyMachine.data, {
    sellerFeeBasisPoints: 1000,
    isMutable: false,
    configLineSettings: data.configLineSettings,
  });

  data.configLineSettings.nameLength = 15;
  data.configLineSettings.uriLength = 100;
  // should fail since length is greater than the original allocated value
  const { tx: updateTransaction2 } = await init.updateCandyMachine(
    t,
    address,
    payerPair,
    data,
    fstTxHandler,
  );
  await updateTransaction2.assertError(t);

  data.itemsAvailable = 100;
  data.configLineSettings.nameLength = 5;
  data.configLineSettings.uriLength = 10;
  // should fail since it is not possible to change the itemsAvailable when
  // config lines are used
  const { tx: updateTransaction3 } = await init.updateCandyMachine(
    t,
    address,
    payerPair,
    data,
    fstTxHandler,
  );
  await updateTransaction3.assertError(t);
});

test('update (hidden settings)', async (t) => {
  const { fstTxHandler, payerPair, connection } = await init.payer();
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
      name: 'Hidden NFT',
      uri: 'https://arweave.net/uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
      hash: Buffer.from('74bac30d82a0baa41dd2bee4b41bbc36').toJSON().data,
    },
  };

  const { tx: transaction, candyMachine: address } = await init.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyMachine = await CandyMachine.fromAccountAddress(connection, address);

  spok(t, candyMachine.data, {
    sellerFeeBasisPoints: 500,
    isMutable: true,
    hiddenSettings: data.hiddenSettings,
  });

  data.itemsAvailable = 1000;

  const { tx: updateTransaction1 } = await init.updateCandyMachine(
    t,
    address,
    payerPair,
    data,
    fstTxHandler,
  );
  await updateTransaction1.assertSuccess(t);
  const updatedCandyMachine = await CandyMachine.fromAccountAddress(connection, address);

  spok(t, updatedCandyMachine.data, {
    itemsAvailable: spokSameBignum(1000),
  });

  const updatedData: program.CandyMachineData = {
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
  // should fail since length is greater than the original allocated value
  const { tx: updateTransaction2 } = await init.updateCandyMachine(
    t,
    address,
    payerPair,
    updatedData,
    fstTxHandler,
  );
  await updateTransaction2.assertError(t);
});
