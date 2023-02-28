import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { AccountVersion, CandyMachine, CandyMachineData } from '../src/generated';
import { TokenStandard } from '@metaplex-foundation/mpl-token-metadata';
import spok from 'spok';

killStuckProcess();

test('set token standard: NFT -> pNFT', async (t) => {
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

  const { tx: txInit, candyMachine } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  await txInit.assertSuccess(t);

  let candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);
  spok(t, candyMachineObject, {
    version: AccountVersion.V1,
    tokenStandard: TokenStandard.NonFungible,
  });

  const { tx: txTokenStandard } = await API.setTokenStandard(
    t,
    payerPair,
    candyMachine,
    candyMachineObject,
    payerPair,
    TokenStandard.ProgrammableNonFungible,
    fstTxHandler,
    connection,
  );
  await txTokenStandard.assertSuccess(t);

  candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);
  spok(t, candyMachineObject, {
    version: AccountVersion.V2,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
  });
});

test('set token standard: NFT -> pNFT -> NFT', async (t) => {
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

  const { tx: txInit, candyMachine } = await API.initialize(
    t,
    payerPair,
    data,
    fstTxHandler,
    connection,
  );
  await txInit.assertSuccess(t);

  let candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);
  spok(t, candyMachineObject, {
    version: AccountVersion.V1,
    tokenStandard: TokenStandard.NonFungible,
  });

  // to pNFT
  const { tx: txpNft } = await API.setTokenStandard(
    t,
    payerPair,
    candyMachine,
    candyMachineObject,
    payerPair,
    TokenStandard.ProgrammableNonFungible,
    fstTxHandler,
    connection,
  );
  await txpNft.assertSuccess(t);

  candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);
  spok(t, candyMachineObject, {
    version: AccountVersion.V2,
    tokenStandard: TokenStandard.ProgrammableNonFungible,
  });

  const { tx: txNFT } = await API.setTokenStandard(
    t,
    payerPair,
    candyMachine,
    candyMachineObject,
    payerPair,
    TokenStandard.NonFungible,
    fstTxHandler,
    connection,
  );
  await txNFT.assertSuccess(t);

  candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);
  spok(t, candyMachineObject, {
    version: AccountVersion.V2,
    tokenStandard: TokenStandard.NonFungible,
  });
});
