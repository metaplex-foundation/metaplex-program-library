import test from 'tape';
import spok from 'spok';
import { InitTransactions, killStuckProcess } from './setup';
import { AccountVersion, CandyMachine, CandyMachineData } from '../src/generated';
import { spokSameBignum, spokSamePubkey } from './utils';
import { TokenStandard } from '@metaplex-foundation/mpl-token-metadata';

killStuckProcess();

test('initialize: new candy machine (pNFT)', async (t) => {
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

  const { tx: transaction, candyMachine: address } = await API.initializeV2(
    t,
    payerPair,
    data,
    TokenStandard.ProgrammableNonFungible,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyMachine = await CandyMachine.fromAccountAddress(connection, address);

  spok(t, candyMachine, {
    version: AccountVersion.V2,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
    authority: spokSamePubkey(payerPair.publicKey),
    itemsRedeemed: spokSameBignum(0),
    data: {
      itemsAvailable: spokSameBignum(items),
      maxSupply: spokSameBignum(0),
      isMutable: true,
      creators: data.creators,
      configLineSettings: data.configLineSettings,
    },
  });
  // hidden settings must be null
  t.notOk(candyMachine.data.hiddenSettings, 'hidden settings should be null');
});

test('initialize: new candy machine (hidden settings)', async (t) => {
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
    configLineSettings: null,
    hiddenSettings: {
      name: 'Hidden NFT',
      uri: 'https://arweave.net/uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
      hash: Buffer.from('74bac30d82a0baa41dd2bee4b41bbc36').toJSON().data,
    },
  };

  const { tx: transaction, candyMachine: address } = await API.initializeV2(
    t,
    payerPair,
    data,
    TokenStandard.NonFungible,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertSuccess(t);
  // retrieves the created candy machine
  const candyMachine = await CandyMachine.fromAccountAddress(connection, address);

  spok(t, candyMachine, {
    version: AccountVersion.V2,
    tokenStandard: TokenStandard.NonFungible,
    authority: spokSamePubkey(payerPair.publicKey),
    itemsRedeemed: spokSameBignum(0),
    data: {
      itemsAvailable: spokSameBignum(items),
      maxSupply: spokSameBignum(0),
      isMutable: true,
      creators: data.creators,
      hiddenSettings: data.hiddenSettings,
    },
  });
  // config lines must be null
  t.notOk(candyMachine.data.configLineSettings, 'config lines settings should be null');
});

test('initialize: new pNFT candy machine (config line + hidden settings)', async (t) => {
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
    hiddenSettings: {
      name: 'Hidden NFT',
      uri: 'https://arweave.net/uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
      hash: Buffer.from('74bac30d82a0baa41dd2bee4b41bbc36').toJSON().data,
    },
  };

  const { tx: transaction } = await API.initializeV2(
    t,
    payerPair,
    data,
    TokenStandard.ProgrammableNonFungible,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertError(t, /hidden uris do not have config lines/i);
});

test('initialize: new candy machine (no config line + no hidden settings)', async (t) => {
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
    hiddenSettings: null,
  };

  const { tx: transaction } = await API.initializeV2(
    t,
    payerPair,
    data,
    TokenStandard.NonFungible,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await transaction.assertError(t, /Missing config lines settings/i);
});
