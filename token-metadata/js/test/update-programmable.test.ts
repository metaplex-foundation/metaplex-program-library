import spok from 'spok';
import { Metadata, TokenStandard } from '../src/generated';
import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { createDefaultAsset } from './utils/DigitalAssetManager';
import { UpdateTestData } from './utils/UpdateTestData';

killStuckProcess();

test('Update: Update a ProgrammableNonFungible With No Config', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();
  const daManager = await createDefaultAsset(
    t,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );
  const { mint, metadata, masterEdition } = daManager;

  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 500,
    creators: null,
  };

  const { tx: updateTx } = await API.update(
    t,
    payer,
    mint,
    metadata,
    masterEdition,
    updateData,
    handler,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);
  spok(t, updatedMetadata.data, {
    sellerFeeBasisPoints: updateData.data.sellerFeeBasisPoints,
    creators: updateData.data.creators,
  });

  t.equal(updatedMetadata.data.name.replace(/\0+/, ''), updateData.data.name);
  t.equal(updatedMetadata.data.symbol.replace(/\0+/, ''), updateData.data.symbol);
  t.equal(updatedMetadata.data.uri.replace(/\0+/, ''), updateData.data.uri);
});

// test('Update: Update a ProgrammableNonFungible With Config Set', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

//   const daManager = await createDefaultAsset(
//     t,
//     API,
//     handler,
//     payer,
//     TokenStandard.ProgrammableNonFungible,
//   );
//   const { mint, metadata, masterEdition } = daManager;

//   const updateData = new UpdateTestData();
//   updateData.data = {
//     name: 'new-name',
//     symbol: 'new-symbol',
//     uri: 'new-uri',
//     sellerFeeBasisPoints: 500,
//     creators: null,
//   };

//   const { tx: updateTx } = await API.update(
//     t,
//     payer,
//     mint,
//     metadata,
//     masterEdition,
//     updateData,
//     handler,
//   );
//   await updateTx.assertSuccess(t);

//   const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);
//   spok(t, updatedMetadata.data, {
//     sellerFeeBasisPoints: updateData.data.sellerFeeBasisPoints,
//     creators: updateData.data.creators,
//   });

//   t.equal(updatedMetadata.data.name.replace(/\0+/, ''), updateData.data.name);
//   t.equal(updatedMetadata.data.symbol.replace(/\0+/, ''), updateData.data.symbol);
//   t.equal(updatedMetadata.data.uri.replace(/\0+/, ''), updateData.data.uri);
// });

// test('Update: Update a ProgrammableNonFungible Where User Adds Rules', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();
//   const daManager = await createDefaultAsset(
//     t,
//     API,
//     handler,
//     payer,
//     TokenStandard.ProgrammableNonFungible,
//   );
//   const { mint, metadata, masterEdition } = daManager;

//   const updateData = new UpdateTestData();
//   updateData.data = {
//     name: 'new-name',
//     symbol: 'new-symbol',
//     uri: 'new-uri',
//     sellerFeeBasisPoints: 500,
//     creators: null,
//   };

//   const { tx: updateTx } = await API.update(
//     t,
//     payer,
//     mint,
//     metadata,
//     masterEdition,
//     updateData,
//     handler,
//   );
//   await updateTx.assertSuccess(t);

//   const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);
//   spok(t, updatedMetadata.data, {
//     sellerFeeBasisPoints: updateData.data.sellerFeeBasisPoints,
//     creators: updateData.data.creators,
//   });

//   t.equal(updatedMetadata.data.name.replace(/\0+/, ''), updateData.data.name);
//   t.equal(updatedMetadata.data.symbol.replace(/\0+/, ''), updateData.data.symbol);
//   t.equal(updatedMetadata.data.uri.replace(/\0+/, ''), updateData.data.uri);
// });
