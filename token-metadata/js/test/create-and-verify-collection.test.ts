import test from 'tape';

import {
  ApproveCollectionAuthority,
  DataV2,
  MetadataProgram,
  VerifyCollection,
} from '../src/mpl-token-metadata';
import {
  killStuckProcess,
  getMetadataData,
  URI,
  NAME,
  SYMBOL,
  connectionURL,
  SELLER_FEE_BASIS_POINTS,
  logDebug,
} from './utils';
import { airdrop, PayerTransactionHandler } from '@metaplex-foundation/amman';
import { Connection, Keypair } from '@solana/web3.js';
import { createCollection, createMasterEdition } from './actions';
import { Collection } from '../src/accounts';
import { SetAndVerifyCollectionCollection } from '../src/transactions';

killStuckProcess();
// -----------------
// Success Cases
// -----------------
test('verify-collection', async (t) => {
  const payer = Keypair.generate();
  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const collectionNft = await createCollection(connection, transactionHandler, payer);

  const initMetadataData = new DataV2({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
    collection: new Collection({ key: collectionNft.mint.publicKey.toBase58(), verified: false }),
    uses: null,
  });
  const collectionMemberNft = await createMasterEdition(
    connection,
    transactionHandler,
    payer,
    initMetadataData,
    0,
  );
  console.log('collectionMemberNft', collectionMemberNft.metadata.toBase58());
  const updatedMetadataBeforeVerification = await getMetadataData(
    connection,
    collectionMemberNft.metadata,
  );
  t.ok(updatedMetadataBeforeVerification.collection, 'collection should be not null');
  t.not(
    updatedMetadataBeforeVerification.collection?.verified,
    'collection should be not be verified',
  );
  const collectionVerifyCollectionTransaction = new VerifyCollection(
    { feePayer: payer.publicKey },
    {
      metadata: collectionMemberNft.metadata,
      collectionAuthority: payer.publicKey,
      collectionMint: collectionNft.mint.publicKey,
      collectionMetadata: collectionNft.metadata,
      collectionMasterEdition: collectionNft.masterEditionPubkey,
    },
  );
  await transactionHandler.sendAndConfirmTransaction(collectionVerifyCollectionTransaction, [
    payer,
  ]);
  const updatedMetadataAfterVerification = await getMetadataData(
    connection,
    collectionMemberNft.metadata,
  );
  t.ok(updatedMetadataAfterVerification.collection, 'collection should be not null');
  t.ok(updatedMetadataAfterVerification.collection?.verified, 'collection should be verified');
});

// -----------------
// Success Cases
// -----------------
test('set-and-verify-collection', async (t) => {
  const payer = Keypair.generate();
  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const collectionNft = await createCollection(connection, transactionHandler, payer);

  const initMetadataData = new DataV2({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
    collection: null,
    uses: null,
  });
  const collectionMemberNft = await createMasterEdition(
    connection,
    transactionHandler,
    payer,
    initMetadataData,
    0,
  );

  const updatedMetadataBeforeVerification = await getMetadataData(
    connection,
    collectionMemberNft.metadata,
  );

  t.not(updatedMetadataBeforeVerification.collection, 'collection should be null');

  const collectionVerifyCollectionTransaction = new SetAndVerifyCollectionCollection(
    { feePayer: payer.publicKey },
    {
      metadata: collectionMemberNft.metadata,
      collectionAuthority: payer.publicKey,
      updateAuthority: payer.publicKey,
      collectionMint: collectionNft.mint.publicKey,
      collectionMetadata: collectionNft.metadata,
      collectionMasterEdition: collectionNft.masterEditionPubkey,
    },
  );
  const txDetails = await transactionHandler.sendAndConfirmTransaction(
    collectionVerifyCollectionTransaction,
    [payer],
    { skipPreflight: true },
  );
  logDebug(txDetails.txSummary.logMessages.join('\n'));
  const updatedMetadataAfterVerification = await getMetadataData(
    connection,
    collectionMemberNft.metadata,
  );

  t.ok(updatedMetadataAfterVerification.collection, 'collection should be not null');

  t.ok(updatedMetadataAfterVerification.collection?.verified, 'collection should be verified');
});
test('Delegated Authority', (t) => {
  t.test('Fail: Verify Collection', async (t) => {
    const payer = Keypair.generate();
    const connection = new Connection(connectionURL, 'confirmed');
    const transactionHandler = new PayerTransactionHandler(connection, payer);

    await airdrop(connection, payer.publicKey, 2);
    const collectionNft = await createCollection(connection, transactionHandler, payer);

    const initMetadataData = new DataV2({
      uri: URI,
      name: NAME,
      symbol: SYMBOL,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
      creators: null,
      collection: new Collection({ key: collectionNft.mint.publicKey.toBase58(), verified: false }),
      uses: null,
    });
    const collectionMemberNft = await createMasterEdition(
      connection,
      transactionHandler,
      payer,
      initMetadataData,
      0,
    );

    const updatedMetadataBeforeVerification = await getMetadataData(
      connection,
      collectionMemberNft.metadata,
    );

    t.ok(updatedMetadataBeforeVerification.collection, 'collection should be null');
    t.false(updatedMetadataBeforeVerification.collection?.verified, 'collection cant be verified');
    const delegatedAuthority = Keypair.generate();
    await airdrop(connection, delegatedAuthority.publicKey, 2);
    const dARecord = await MetadataProgram.findCollectionAuthorityAccount(
      collectionNft.mint.publicKey,
      delegatedAuthority.publicKey,
    );
    const collectionVerifyCollectionTransaction = new VerifyCollection(
      { feePayer: payer.publicKey },
      {
        metadata: collectionMemberNft.metadata,
        collectionAuthority: delegatedAuthority.publicKey,
        collectionMint: collectionNft.mint.publicKey,
        collectionMetadata: collectionNft.metadata,
        collectionMasterEdition: collectionNft.masterEditionPubkey,
        collectionAuthorityRecord: dARecord[0],
      },
    );
    const txDetails = await transactionHandler.sendAndConfirmTransaction(
      collectionVerifyCollectionTransaction,
      [delegatedAuthority],
      { skipPreflight: true },
    );
    logDebug(txDetails.txSummary.logMessages.join('\n'));
    t.deepEqual(
      txDetails.txSummary.err,
      { InstructionError: [0, { Custom: 81 }] },
      'Collection Update Authority is invalid',
    );
  });
  t.test('Fail: Set and Verify Collection', async (t) => {
    const payer = Keypair.generate();
    const connection = new Connection(connectionURL, 'confirmed');
    const transactionHandler = new PayerTransactionHandler(connection, payer);

    await airdrop(connection, payer.publicKey, 2);

    const collectionNft = await createCollection(connection, transactionHandler, payer);

    const initMetadataData = new DataV2({
      uri: URI,
      name: NAME,
      symbol: SYMBOL,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
      creators: null,
      collection: null,
      uses: null,
    });
    const collectionMemberNft = await createMasterEdition(
      connection,
      transactionHandler,
      payer,
      initMetadataData,
      0,
    );

    const updatedMetadataBeforeVerification = await getMetadataData(
      connection,
      collectionMemberNft.metadata,
    );

    t.notOk(updatedMetadataBeforeVerification.collection, 'collection should be null');
    const delegatedAuthority = Keypair.generate();
    await airdrop(connection, delegatedAuthority.publicKey, 2);
    const dARecord = await MetadataProgram.findCollectionAuthorityAccount(
      collectionNft.mint.publicKey,
      delegatedAuthority.publicKey,
    );

    const collectionVerifyCollectionTransaction = new SetAndVerifyCollectionCollection(
      { feePayer: delegatedAuthority.publicKey },
      {
        metadata: collectionMemberNft.metadata,
        collectionAuthority: delegatedAuthority.publicKey,
        updateAuthority: payer.publicKey,
        collectionMint: collectionNft.mint.publicKey,
        collectionMetadata: collectionNft.metadata,
        collectionMasterEdition: collectionNft.masterEditionPubkey,
        collectionAuthorityRecord: dARecord[0],
      },
    );
    const txDetails = await transactionHandler.sendAndConfirmTransaction(
      collectionVerifyCollectionTransaction,
      [delegatedAuthority],
      { skipPreflight: true },
    );
    logDebug(txDetails.txSummary.logMessages.join('\n'));
    t.deepEqual(
      txDetails.txSummary.err,
      { InstructionError: [0, { Custom: 81 }] },
      'Collection Update Authority is invalid',
    );
  });
  t.test('Success: Verify Collection', async (t) => {
    const payer = Keypair.generate();
    const connection = new Connection(connectionURL, 'confirmed');
    const transactionHandler = new PayerTransactionHandler(connection, payer);

    await airdrop(connection, payer.publicKey, 2);
    const collectionNft = await createCollection(connection, transactionHandler, payer);

    const initMetadataData = new DataV2({
      uri: URI,
      name: NAME,
      symbol: SYMBOL,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
      creators: null,
      collection: new Collection({ key: collectionNft.mint.publicKey.toBase58(), verified: false }),
      uses: null,
    });
    const collectionMemberNft = await createMasterEdition(
      connection,
      transactionHandler,
      payer,
      initMetadataData,
      0,
    );

    const updatedMetadataBeforeVerification = await getMetadataData(
      connection,
      collectionMemberNft.metadata,
    );

    t.ok(updatedMetadataBeforeVerification.collection, 'collection should not be null');
    t.false(updatedMetadataBeforeVerification.collection?.verified, 'collection cant be verified');
    const delegatedAuthority = Keypair.generate();
    await airdrop(connection, delegatedAuthority.publicKey, 2);
    const dARecord = await MetadataProgram.findCollectionAuthorityAccount(
      collectionNft.mint.publicKey,
      delegatedAuthority.publicKey,
    );

    const approveTransaction = new ApproveCollectionAuthority(
      { feePayer: payer.publicKey },
      {
        collectionAuthorityRecord: dARecord[0],
        newCollectionAuthority: delegatedAuthority.publicKey,
        updateAuthority: payer.publicKey,
        metadata: collectionNft.metadata,
        mint: collectionNft.mint.publicKey,
      },
    );
    const approveTxnDetails = await transactionHandler.sendAndConfirmTransaction(
      approveTransaction,
      [payer],
      { skipPreflight: true },
    );
    logDebug(approveTxnDetails.txSummary.logMessages.join('\n'));
    const collectionVerifyCollectionTransaction = new VerifyCollection(
      { feePayer: payer.publicKey },
      {
        metadata: collectionMemberNft.metadata,
        collectionAuthority: delegatedAuthority.publicKey,
        collectionMint: collectionNft.mint.publicKey,
        collectionMetadata: collectionNft.metadata,
        collectionMasterEdition: collectionNft.masterEditionPubkey,
        collectionAuthorityRecord: dARecord[0],
      },
    );
    const txDetails = await transactionHandler.sendAndConfirmTransaction(
      collectionVerifyCollectionTransaction,
      [delegatedAuthority],
      { skipPreflight: true },
    );
    logDebug(txDetails.txSummary.logMessages.join('\n'));
    const updatedMetadataAfterVerification = await getMetadataData(
      connection,
      collectionMemberNft.metadata,
    );
    t.ok(updatedMetadataAfterVerification.collection, 'collection should not be null');
    t.true(updatedMetadataAfterVerification.collection?.verified, 'collection is verified');
  });

  t.test('Success: Set and Verify Collection', async (t) => {
    const payer = Keypair.generate();
    const connection = new Connection(connectionURL, 'confirmed');
    const transactionHandler = new PayerTransactionHandler(connection, payer);

    await airdrop(connection, payer.publicKey, 2);

    const collectionNft = await createCollection(connection, transactionHandler, payer);

    const initMetadataData = new DataV2({
      uri: URI,
      name: NAME,
      symbol: SYMBOL,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
      creators: null,
      collection: null,
      uses: null,
    });
    const collectionMemberNft = await createMasterEdition(
      connection,
      transactionHandler,
      payer,
      initMetadataData,
      0,
    );

    const updatedMetadataBeforeVerification = await getMetadataData(
      connection,
      collectionMemberNft.metadata,
    );

    t.not(updatedMetadataBeforeVerification.collection, 'collection should be null');
    const delegatedAuthority = Keypair.generate();
    await airdrop(connection, delegatedAuthority.publicKey, 2);
    const dARecord = await MetadataProgram.findCollectionAuthorityAccount(
      collectionNft.mint.publicKey,
      delegatedAuthority.publicKey,
    );

    const approveTransaction = new ApproveCollectionAuthority(
      { feePayer: payer.publicKey },
      {
        collectionAuthorityRecord: dARecord[0],
        newCollectionAuthority: delegatedAuthority.publicKey,
        updateAuthority: payer.publicKey,
        metadata: collectionNft.metadata,
        mint: collectionNft.mint.publicKey,
      },
    );

    const approveTxnDetails = await transactionHandler.sendAndConfirmTransaction(
      approveTransaction,
      [payer],
      { skipPreflight: true },
    );
    logDebug(approveTxnDetails.txSummary.logMessages.join('\n'));

    const collectionVerifyCollectionTransaction = new SetAndVerifyCollectionCollection(
      { feePayer: delegatedAuthority.publicKey },
      {
        metadata: collectionMemberNft.metadata,
        collectionAuthority: delegatedAuthority.publicKey,
        updateAuthority: payer.publicKey,
        collectionMint: collectionNft.mint.publicKey,
        collectionMetadata: collectionNft.metadata,
        collectionMasterEdition: collectionNft.masterEditionPubkey,
        collectionAuthorityRecord: dARecord[0],
      },
    );
    await transactionHandler.sendAndConfirmTransaction(
      collectionVerifyCollectionTransaction,
      [delegatedAuthority],
      { skipPreflight: true },
    );
    const updatedMetadataAfterVerification = await getMetadataData(
      connection,
      collectionMemberNft.metadata,
    );
    t.ok(updatedMetadataAfterVerification.collection, 'collection should not be null');
    t.true(updatedMetadataAfterVerification.collection?.verified, 'collection is verified');
  });
});
