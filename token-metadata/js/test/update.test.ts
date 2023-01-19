import spok from 'spok';
import {
  AssetData,
  Data,
  DelegateArgs,
  Metadata,
  PROGRAM_ID,
  TokenRecord,
  TokenStandard,
  TokenState,
} from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { Keypair, PublicKey } from '@solana/web3.js';
import { createAndMintDefaultAsset, createDefaultAsset } from './utils/digital-asset-manager';
import { UpdateTestData } from './utils/update-test-data';
import { encode } from '@msgpack/msgpack';
import { PROGRAM_ID as TOKEN_AUTH_RULES_ID } from '@metaplex-foundation/mpl-token-auth-rules';
import { spokSamePubkey } from './utils';
import { findTokenRecordPda } from './utils/programmable';

killStuckProcess();

test('Update: NonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;
  const assetData = await daManager.getAssetData(connection);

  const authority = payer;

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

  const updateData = new UpdateTestData();
  updateData.data = data;
  updateData.authorizationData = authorizationData;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
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

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

  const updateData = new UpdateTestData();
  updateData.data = data;
  updateData.authorizationData = authorizationData;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
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

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

  const updateData = new UpdateTestData();
  updateData.data = data;
  updateData.authorizationData = authorizationData;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertSuccess(t);
});

test('Update: Cannot Flip IsMutable to True', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

  // Flip isMutable to false
  const updateData = new UpdateTestData();
  updateData.isMutable = false;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx2.assertError(t, /Is Mutable can only be flipped to false/i);
});

test('Update: Cannot Flip PrimarySaleHappened to False', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

  // Flip to true
  const updateData = new UpdateTestData();
  updateData.primarySaleHappened = true;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx2.assertError(t, /Primary sale can only be flipped to true/i);
});

test('Update: Set New Update Authority', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;
  const newUpdateAuthority = new Keypair().publicKey;

  // Flip to true
  const updateData = new UpdateTestData();
  updateData.newUpdateAuthority = newUpdateAuthority;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
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

  // Flip isMutable to false
  const updateData = new UpdateTestData();
  updateData.isMutable = false;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx2.assertError(t, /Data is immutable/i);
});

test('Update: Name Cannot Exceed 32 Bytes', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /Name too long/i);
});

test('Update: Symbol Cannot Exceed 10 Bytes', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /Symbol too long/i);
});

test('Update: URI Cannot Exceed 200 Bytes', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /Uri too long/i);
});

test('Update: SellerFeeBasisPoints Cannot Exceed 10_000', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /Basis points cannot be more than 10000/i);
});

test('Update: Creators Array Cannot Exceed Five Items', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /Creators list too long/i);
});

test('Update: No Duplicate Creator Addresses', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /No duplicate creator addresses/i);
});

test('Update: Creator Shares Must Equal 100', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

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
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /Share total must equal 100 for creator array/i);
});

test('Update: Cannot Unverify Another Creator', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

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
    authority,
    updateData,
    null,
    masterEdition,
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
    authority,
    updateData,
    null,
    masterEdition,
  );

  await updateTx2.assertError(t, /cannot unilaterally unverify another creator/i);
});

test('Update: Cannot Verify Another Creator', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  const authority = payer;

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
    authority,
    updateData,
    null,
    masterEdition,
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
    authority,
    updateData2,
    null,
    masterEdition,
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
    tokenStandard: TokenStandard.NonFungible,
    collection: { key: collectionParent.publicKey, verified: false },
    uses: null,
    collectionDetails: null,
    ruleSet: null,
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
  updateData.collection = {
    __kind: 'Set',
    fields: [
      {
        key: newCollectionParent.publicKey,
        verified: false,
      },
    ],
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  spok(t, updatedMetadata.collection, {
    verified: false,
    key: spokSamePubkey(newCollectionParent.publicKey),
  });
});

test('Update: Fail to Verify an Unverified Collection', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const name = 'DigitalAsset';
  const symbol = 'DA';
  const uri = 'uri';

  const authority = payer;

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
    tokenStandard: TokenStandard.NonFungible,
    collection: { key: collectionParent.publicKey, verified: false },
    uses: null,
    collectionDetails: null,
    ruleSet: null,
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
  updateData.collection = {
    __kind: 'Set',
    fields: [
      {
        key: collectionParent.publicKey,
        verified: true,
      },
    ],
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
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
    tokenStandard: TokenStandard.NonFungible,
    collection: { key: collectionMint, verified: false },
    uses: null,
    collectionDetails: null,
    ruleSet: null,
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
  updateData.collection = {
    __kind: 'Set',
    fields: [
      {
        key: newCollectionParent.publicKey,
        verified: true,
      },
    ],
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /Collection cannot be verified in this instruction/);
});

test('Update: Update pNFT Config', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    null,
    1,
  );

  const authority = payer;
  const dummyRuleSet = Keypair.generate().publicKey;

  const updateData = new UpdateTestData();
  updateData.ruleSet = {
    __kind: 'Set',
    fields: [dummyRuleSet],
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  spok(t, updatedMetadata.programmableConfig, {
    ruleSet: dummyRuleSet,
  });
});

test('Update: Fail to update rule set on NFT', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const authority = payer;
  const dummyRuleSet = Keypair.generate().publicKey;

  const { mint, metadata, masterEdition } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
    null,
    1,
  );

  const updateData = new UpdateTestData();
  updateData.ruleSet = {
    __kind: 'Set',
    fields: [dummyRuleSet],
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /Invalid token standard/);
});

test('Update: Update existing pNFT rule set config to None', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const authority = payer;

  // We need a real ruleset here to pass the mint checks.
  // Set up our rule set
  const ruleSetName = 'update_test';
  const ruleSet = {
    version: 1,
    ruleSetName: ruleSetName,
    owner: Array.from(authority.publicKey.toBytes()),
    operations: {
      Transfer: {
        PubkeyMatch: {
          pubkey: Array.from(authority.publicKey.toBytes()),
          field: 'Target',
        },
      },
    },
  };
  const serializedRuleSet = encode(ruleSet);

  // Find the ruleset PDA
  const [ruleSetPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('rule_set'), payer.publicKey.toBuffer(), Buffer.from(ruleSetName)],
    TOKEN_AUTH_RULES_ID,
  );

  const { tx: createRuleSetTx } = await API.createRuleSet(
    t,
    payer,
    ruleSetPda,
    serializedRuleSet,
    handler,
  );
  await createRuleSetTx.assertSuccess(t);

  const { mint, metadata, masterEdition } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    ruleSetPda,
    1,
  );

  const updateData = new UpdateTestData();
  updateData.ruleSet = {
    __kind: 'Clear',
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
    null,
    ruleSetPda,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  t.equal(updatedMetadata.programmableConfig, null);
});

test('Update: Invalid Update Authority Fails', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

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
    invalidUpdateAuthority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertError(t, /Update Authority given does not match/);
});

test('Update: Delegate Authority Type Not Supported', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');
  // delegate PDA
  const [delegateRecord] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('metadata'),
      PROGRAM_ID.toBuffer(),
      daManager.mint.toBuffer(),
      Buffer.from('update_delegate'),
      payer.publicKey.toBuffer(),
      delegate.publicKey.toBuffer(),
    ],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Delegate Record', delegateRecord);

  const args: DelegateArgs = {
    __kind: 'UpdateV1',
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate.publicKey,
    daManager.mint,
    daManager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    delegateRecord,
    daManager.masterEdition,
  );
  await delegateTx.assertSuccess(t);

  const assetData = await daManager.getAssetData(connection);
  const authority = delegate;

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };
  const authorizationData = daManager.emptyAuthorizationData();

  const updateData = new UpdateTestData();
  updateData.data = data;
  updateData.authorizationData = authorizationData;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    delegateRecord,
    masterEdition,
  );

  updateTx.then((x) =>
    x.assertLogs(t, [/Authority type: Delegate/i, /Feature not supported currently/i], {
      txLabel: 'tx: Update',
    }),
  );
  await updateTx.assertError(t);
});

test('Update: Holder Authority Type Not Supported', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const daManager = await createDefaultAsset(t, connection, API, handler, payer);
  const { mint, metadata, masterEdition } = daManager;

  // initialize a token account

  const [, holder] = await amman.genLabeledKeypair('Holder');

  const { tx: tokenTx, token } = await API.createTokenAccount(
    mint,
    payer,
    connection,
    handler,
    holder.publicKey,
  );
  await tokenTx.assertSuccess(t);

  // mint 1 asset

  const amount = 1;

  const { tx: mintTx } = await API.mint(
    t,
    connection,
    payer,
    mint,
    metadata,
    masterEdition,
    daManager.emptyAuthorizationData(),
    amount,
    handler,
    token,
  );
  await mintTx.assertSuccess(t);

  const assetData = await daManager.getAssetData(connection);

  // Change some values and run update.
  const data: Data = {
    name: 'DigitalAsset2',
    symbol: 'DA2',
    uri: 'uri2',
    sellerFeeBasisPoints: 0,
    creators: assetData.creators,
  };

  const authorizationData = daManager.emptyAuthorizationData();

  const updateData = new UpdateTestData();
  updateData.data = data;
  updateData.authorizationData = authorizationData;

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    holder,
    updateData,
    null,
    masterEdition,
    token,
  );

  updateTx.then((x) =>
    x.assertLogs(t, [/Authority type: Holder/i, /Feature not supported currently/i], {
      txLabel: 'tx: Update',
    }),
  );
  await updateTx.assertError(t);
});

test('Update: Update pNFT Config with locked token', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    null,
    1,
  );

  // token record PDA
  const tokenRecord = findTokenRecordPda(mint, payer.publicKey);
  amman.addr.addLabel('Token Record', tokenRecord);

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Unlocked /* asset should be unlocked */,
  });

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');

  const args: DelegateArgs = {
    __kind: 'UtilityV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate.publicKey,
    mint,
    metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    masterEdition,
    token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // lock asset with delegate

  const { tx: lockTx } = await API.lock(
    delegate,
    mint,
    metadata,
    token,
    payer,
    handler,
    tokenRecord,
    null,
    masterEdition,
  );
  await lockTx.assertSuccess(t);

  // updates the metadata

  const authority = payer;
  const dummyRuleSet = Keypair.generate().publicKey;

  const updateData = new UpdateTestData();
  updateData.ruleSet = {
    __kind: 'Set',
    fields: [dummyRuleSet],
  };

  const { tx: updateTx } = await API.update(
    t,
    handler,
    mint,
    metadata,
    authority,
    updateData,
    null,
    masterEdition,
  );
  await updateTx.assertSuccess(t);

  const updatedMetadata = await Metadata.fromAccountAddress(connection, metadata);

  spok(t, updatedMetadata.programmableConfig, {
    ruleSet: dummyRuleSet,
  });
});
