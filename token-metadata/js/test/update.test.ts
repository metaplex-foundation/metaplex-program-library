import spok from 'spok';
import { AssetData, AuthorizationData, Data, Metadata, TokenStandard } from '../src/generated';
import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { Connection, Keypair } from '@solana/web3.js';
import { DigitalAssetManager } from './utils/DigitalAssetManager';
import { PayerTransactionHandler } from '@metaplex-foundation/amman-client';

killStuckProcess();

async function createDefaultAsset(
  t: test.Test,
  API: InitTransactions,
  connection: Connection,
  handler: PayerTransactionHandler,
  payer: Keypair,
): Promise<DigitalAssetManager> {
  const name = 'DigitalAsset';
  const symbol = 'DA';
  const uri = 'uri';

  // Create the initial asset and ensure it was created successfully
  const assetData: AssetData = {
    name,
    symbol,
    uri,
    sellerFeeBasisPoints: 0,
    updateAuthority: payer.publicKey,
    creators: [
      {
        address: payer.publicKey,
        share: 100,
        verified: false,
      },
    ],
    primarySaleHappened: false,
    isMutable: true,
    editionNonce: null,
    tokenStandard: TokenStandard.NonFungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    programmableConfig: null,
    delegateState: null,
  };

  const {
    tx: createTx,
    mint: mint,
    metadata: metadata,
    masterEdition: masterEdition,
  } = await API.create(t, payer, assetData, 0, 0, handler);
  await createTx.assertSuccess(t);

  const md = await Metadata.fromAccountAddress(connection, metadata);

  spok(t, md, {
    data: {
      sellerFeeBasisPoints: 0,
    },
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.NonFungible,
  });

  t.equal(md.data.name.replace(/\0+/, ''), name);
  t.equal(md.data.symbol.replace(/\0+/, ''), symbol);
  t.equal(md.data.uri.replace(/\0+/, ''), uri);

  const daManager = new DigitalAssetManager(mint, metadata, masterEdition);

  return daManager;
}

test('Update: NonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, API, connection, handler, payer);
  const { mint, metadata, masterEdition } = daManager;
  const assetData = await daManager.getAssetData(connection);

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData: AuthorizationData = {
    derivedKeySeeds: null,
    leafInfo: null,
    name: 'rule-name',
  };

  const updateData = {
    newUpdateAuthority: null,
    data: data,
    primarySaleHappened: null,
    isMutable: null,
    tokenStandard: null,
    collection: null,
    uses: null,
    collectionDetails: null,
    programmableConfig: null,
    delegateState: null,
    authorizationData: authorizationData,
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

  spok(t, updatedMetadata, {
    data: {
      sellerFeeBasisPoints: 0,
    },
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.NonFungible,
  });

  t.equal(updatedMetadata.data.name.replace(/\0+/, ''), data.name);
  t.equal(updatedMetadata.data.symbol.replace(/\0+/, ''), data.symbol);
  t.equal(updatedMetadata.data.uri.replace(/\0+/, ''), data.uri);
});

// test('Update: Cannot Flip IsMutable to True', async (t) => {
//   const API = new InitTransactions();
//   const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

//   const daManager = await createDefaultAsset(t, API, connection, handler, payer);
//   const { mint, metadata, masterEdition } = daManager;
//   const assetData = await daManager.getAssetData(connection);

//   const authorizationData = {
//     derivedKeySeeds: null,
//     leafInfo: null,
//     name: 'rule-name',
//   };

//   // Flip isMutable to false
//   assetData.isMutable = false;

//   const { tx: updateTx } = await API.update(
//     t,
//     payer,
//     mint,
//     metadata,
//     masterEdition,
//     assetData,
//     authorizationData,
//     handler,
//   );
//   await updateTx.assertSuccess(t);

//   const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

//   spok(t, updatedMetadata, {
//     isMutable: false,
//   });

//   // Try to flip isMutable to true
//   assetData.isMutable = true;

//   const { tx: updateTx2 } = await API.update(
//     t,
//     payer,
//     mint,
//     metadata,
//     masterEdition,
//     assetData,
//     authorizationData,
//     handler,
//   );
//   await updateTx2.assertError(t, /Is Mutable can only be flipped to false/i);
// });
