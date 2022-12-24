import spok from 'spok';
import { AssetData, AuthorityType, Data, Metadata, TokenStandard } from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { Keypair } from '@solana/web3.js';
import { createAndMintDefaultAsset, createDefaultAsset } from './utils/DigitalAssetManager';
import { UpdateTestData } from './utils/UpdateTestData';

killStuckProcess();

test('Update: NonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;
  const assetData = await daManager.getAssetData(connection);

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

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
    authorizationData,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);
});

test('Update: Fungible Token', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.Fungible,
    null,
    10,
  );
  const { mint, metadata, masterEdition } = daManager;

  const assetData = await daManager.getAssetData(connection);

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

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
    authorizationData,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);
});

test('Update: Fungible Asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.FungibleAsset,
    null,
    10,
  );
  const { mint, metadata, masterEdition } = daManager;

  const assetData = await daManager.getAssetData(connection);

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

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
    authorizationData,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);
});

test('Update: Cannot Flip IsMutable to True', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  // Flip isMutable to false
  const updateData = new UpdateTestData();
  updateData.isMutable = false;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  spok(t, updatedMetadata, {
    isMutable: false,
  });

  // Try to flip isMutable to true
  updateData.isMutable = true;

  const { tx: updateTx2 } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx2.assertError(t, /Is Mutable can only be flipped to false/i);
});

test('Update: Cannot Flip PrimarySaleHappened to False', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  // Flip to true
  const updateData = new UpdateTestData();
  updateData.primarySaleHappened = true;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  spok(t, updatedMetadata, {
    primarySaleHappened: true,
  });

  // Try to flip false -- this should fail
  updateData.primarySaleHappened = false;

  const { tx: updateTx2 } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx2.assertError(t, /Primary sale can only be flipped to true/i);
});

test('Update: Set New Update Authority', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;
  const newUpdateAuthority = new Keypair().publicKey;

  // Flip to true
  const updateData = new UpdateTestData();
  updateData.newUpdateAuthority = newUpdateAuthority;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  spok(t, updatedMetadata, {
    updateAuthority: newUpdateAuthority,
  });
});

test('Update: Cannot Update Immutable Data', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  // Flip isMutable to false
  const updateData = new UpdateTestData();
  updateData.isMutable = false;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);

  // Try to write some data.
  updateData.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 500,
    creators: null,
  };

  const { tx: updateTx2 } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx2.assertError(t, /Data is immutable/i);
});

test('Update: Name Cannot Exceed 32 Bytes', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const updateData = new UpdateTestData();
  updateData.data = {
    name: ''.padEnd(33, 'a'),
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 100,
    creators: null,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Name too long/i);
});

test('Update: Symbol Cannot Exceed 10 Bytes', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'new-name',
    symbol: ''.padEnd(11, 'a'),
    uri: 'new-uri',
    sellerFeeBasisPoints: 100,
    creators: null,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Symbol too long/i);
});

test('Update: URI Cannot Exceed 200 Bytes', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: ''.padEnd(201, 'a'),
    sellerFeeBasisPoints: 100,
    creators: null,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Uri too long/i);
});

test('Update: SellerFeeBasisPoints Cannot Exceed 10_000', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 10_005,
    creators: null,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Basis points cannot be more than 10000/i);
});

test('Update: Creators Array Cannot Exceed Five Items', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const creators = [];

  for (let i = 0; i < 6; i++) {
    creators.push({
      address: new Keypair().publicKey,
      verified: false,
      share: i < 5 ? 20 : 0, // Don't exceed 100% share total.
    });
  }

  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 100,
    creators,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Creators list too long/i);
});

test('Update: No Duplicate Creator Addresses', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const creators = [];

  for (let i = 0; i < 2; i++) {
    creators.push({
      address: payer.publicKey,
      verified: true,
      share: 50,
    });
  }

  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 100,
    creators,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /No duplicate creator addresses/i);
});

test('Update: Creator Shares Must Equal 100', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const creators = [];

  creators.push({
    address: payer.publicKey,
    verified: true,
    share: 101,
  });

  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 100,
    creators,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Share total must equal 100 for creator array/i);
});

test('Update: Cannot Unverify Another Creator', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  // Create a new creator with a different keypair.
  const creatorKey = new Keypair();
  await amman.airdrop(connection, creatorKey.publicKey, 1);

  // Add new creator to metadata.
  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 100,
    creators: [
      {
        address: payer.publicKey,
        share: 100,
        verified: false,
      },
      {
        address: creatorKey.publicKey,
        share: 0,
        verified: false,
      },
    ],
  };

  // Update metadata with new creator.
  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);

  // Sign metadata with new creator.
  const { tx: signMetadataTx } = await API.signMetadata(t, creatorKey, metadata, handler);
  await signMetadataTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  t.equal(updatedMetadata.data.creators[1].verified, true);

  // Have the original keypair try to unverify it.
  const newCreators = [];
  newCreators.push({
    address: creatorKey.publicKey,
    verified: false,
    share: 100,
  });

  const updateData2 = new UpdateTestData();
  updateData2.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 100,
    creators: newCreators,
  };

  const { tx: updateTx2 } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData2,
  );

  await updateTx2.assertError(t, /cannot unilaterally unverify another creator/i);
});

test('Update: Cannot Verify Another Creator', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const creatorKey = new Keypair();
  await amman.airdrop(connection, creatorKey.publicKey, 1);

  // Start with an unverified creator
  const creators = [];
  creators.push({
    address: creatorKey.publicKey,
    verified: false,
    share: 100,
  });

  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 100,
    creators,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  spok(t, updatedMetadata.data, {
    creators: updateData.data.creators,
  });

  // Have a different keypair try to verify it.
  const newCreators = [];
  newCreators.push({
    address: creatorKey.publicKey,
    verified: true,
    share: 100,
  });

  const updateData2 = new UpdateTestData();
  updateData2.data = {
    name: 'new-name',
    symbol: 'new-symbol',
    uri: 'new-uri',
    sellerFeeBasisPoints: 100,
    creators: newCreators,
  };

  const { tx: updateTx2 } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData2,
  );

  await updateTx2.assertError(t, /cannot unilaterally verify another creator, they must sign/i);
});

test('Update: Update Unverified Collection Key', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const name = 'DigitalAsset';
  const symbol = 'DA';
  const uri = 'uri';

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const collectionParent = new Keypair();
  const newCollectionParent = new Keypair();

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
    collection: { key: collectionParent.publicKey, verified: false },
    uses: null,
    collectionDetails: null,
    programmableConfig: null,
    delegateState: null,
  };

  const {
    tx: createTx,
    mint,
    metadata,
    masterEdition,
  } = await API.create(t, payer, assetData, 0, 0, handler);
  await createTx.assertSuccess(t);

  const createdMetadata = await Metadata.fromAccountAddress(connection, metadata);
  spok(t, createdMetadata, {
    collection: {
      key: collectionParent.publicKey,
      verified: false,
    },
  });

  const updateData = new UpdateTestData();
  updateData.collection = { key: newCollectionParent.publicKey, verified: false };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  spok(t, updatedMetadata.collection, {
    verified: updateData.collection.verified,
    key: updateData.collection.key,
  });
});

test('Update: Fail to Verify an Unverified Collection', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const name = 'DigitalAsset';
  const symbol = 'DA';
  const uri = 'uri';

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const collectionParent = new Keypair();

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
    collection: { key: collectionParent.publicKey, verified: false },
    uses: null,
    collectionDetails: null,
    programmableConfig: null,
    delegateState: null,
  };

  const {
    tx: createTx,
    mint,
    metadata,
    masterEdition,
  } = await API.create(t, payer, assetData, 0, 0, handler);
  await createTx.assertSuccess(t);

  const createdMetadata = await Metadata.fromAccountAddress(connection, metadata);
  spok(t, createdMetadata, {
    collection: {
      key: collectionParent.publicKey,
      verified: false,
    },
  });

  const updateData = new UpdateTestData();
  updateData.collection = { key: collectionParent.publicKey, verified: true };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Collection cannot be verified in this instruction/);
});

test('Update: Fail to Update a Verified Collection', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const name = 'DigitalAsset';
  const symbol = 'DA';
  const uri = 'uri';

  // Create parent NFT.
  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const {
    mint: collectionMint,
    metadata: collectionMetadata,
    masterEdition: collectionMasterEdition,
  } = daManager;

  const authority = payer;
  const authorityType = AuthorityType.Metadata;

  const newCollectionParent = new Keypair();

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
    collection: { key: collectionMint, verified: false },
    uses: null,
    collectionDetails: null,
    programmableConfig: null,
    delegateState: null,
  };

  const {
    tx: createTx,
    mint,
    metadata,
    masterEdition,
  } = await API.create(t, payer, assetData, 0, 0, handler);
  await createTx.assertSuccess(t);

  const createdMetadata = await Metadata.fromAccountAddress(connection, metadata);
  spok(t, createdMetadata, {
    collection: {
      key: collectionMint,
      verified: false,
    },
  });

  const { tx: verifyCollectionTx } = await API.verifyCollection(
    t,
    payer,
    metadata,
    collectionMint,
    collectionMetadata,
    collectionMasterEdition,
    payer,
    handler,
  );
  await verifyCollectionTx.assertSuccess(t);

  const updateData = new UpdateTestData();
  updateData.collection = { key: newCollectionParent.publicKey, verified: true };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Collection cannot be verified in this instruction/);
});

test('Update: Invalid Update Authority Fails', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authorityType = AuthorityType.Metadata;

  const invalidUpdateAuthority = new Keypair();

  // Flip to true
  const updateData = new UpdateTestData();
  updateData.data = {
    name: 'fake name',
    symbol: 'fake',
    uri: 'fake uri',
    sellerFeeBasisPoints: 500,
    creators: null,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    invalidUpdateAuthority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Update Authority given does not match/);
});

test('Update: Delegate Authority Type Not Supported', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;
  const assetData = await daManager.getAssetData(connection);

  const authority = payer;
  const authorityType = AuthorityType.Delegate;

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

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
    authorizationData,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Feature not supported/);
});

test('Update: Holder Authority Type Not Supported', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;
  const assetData = await daManager.getAssetData(connection);

  const authority = payer;
  const authorityType = AuthorityType.Holder;

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

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
    authorizationData,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Feature not supported/);
});

test('Update: Other Authority Type Not Supported', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;
  const assetData = await daManager.getAssetData(connection);

  const authority = payer;
  const authorityType = AuthorityType.Other;

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

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
    authorizationData,
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    masterEdition,
    authority,
    authorityType,
    updateData,
  );
  await updateTx.assertError(t, /Feature not supported/);
});
