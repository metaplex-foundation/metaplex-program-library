import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { CandyMachine, CandyMachineData } from '../src/generated';
import { COLLECTION_METADATA } from './utils';
import { keypairIdentity, Metaplex } from '@metaplex-foundation/js';

killStuckProcess();

test('set collection', async (t) => {
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

  const { tx: txInit, candyMachine: address } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await txInit.assertSuccess(t);

  // creates a new collection nft
  const metaplex = Metaplex.make(connection).use(keypairIdentity(payerPair));

  const { nft: newCollection } = await metaplex
    .nfts()
    .create({
      uri: COLLECTION_METADATA,
      name: 'CORE Collection',
      sellerFeeBasisPoints: 500,
    })
    .run();

  const candyMachineObject = await CandyMachine.fromAccountAddress(connection, address);

  const { tx: txSet } = await API.setCollection(
    t,
    payerPair,
    address,
    candyMachineObject.collectionMint,
    newCollection,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await txSet.assertSuccess(t);
});

test('set collection: wrong collection mint', async (t) => {
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

  const { tx: txInit, candyMachine: address } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await txInit.assertSuccess(t);

  // creates a new collection nft
  const metaplex = Metaplex.make(connection).use(keypairIdentity(payerPair));

  const { nft: newCollection } = await metaplex
    .nfts()
    .create({
      uri: COLLECTION_METADATA,
      name: 'CORE Collection',
      sellerFeeBasisPoints: 500,
    })
    .run();

  const { tx: txSet } = await API.setCollection(
    t,
    payerPair,
    address,
    newCollection.address,
    newCollection,
    fstTxHandler,
    connection,
  );
  // executes the transaction
  await txSet.assertError(t, /Mint Mismatch/i);
});
