import spok from 'spok';
import {
  CollectionDetails,
  Metadata,
  TokenStandard,
  VerificationArgs,
  VerifyInstructionArgs,
  UnverifyInstructionArgs,
} from '../src/generated';
import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import {
  createAndMintDefaultAsset,
  createAndMintDefaultCollectionParent,
} from './utils/digital-asset-manager';
import { spokSamePubkey } from './utils';

killStuckProcess();

test('Verify and Unverify: creator', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  // Create item NFT.
  const daItemManager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // Creator is set for item but unverified.
  const metadataInitial = await Metadata.fromAccountAddress(connection, daItemManager.metadata);
  const unverifiedCreators = [
    {
      address: payer.publicKey,
      verified: false,
      share: 100,
    },
  ];
  spok(t, metadataInitial.data, {
    creators: unverifiedCreators,
  });

  // Verify.
  const authority = payer;
  const verifyArgs: VerifyInstructionArgs = {
    verificationArgs: VerificationArgs.CreatorV1,
  };

  const { tx: verifyTx } = await API.verify(
    handler,
    authority,
    null,
    daItemManager.metadata,
    null,
    null,
    null,
    verifyArgs,
  );

  await verifyTx.assertSuccess(t);

  // Collection is set for item and verified.
  const metadataVerified = await Metadata.fromAccountAddress(connection, daItemManager.metadata);
  const verifiedCreators = [
    {
      address: payer.publicKey,
      verified: false,
      share: 100,
    },
  ];
  spok(t, metadataVerified.data, {
    creators: verifiedCreators,
  });

  // Unverify.
  const unverifyArgs: UnverifyInstructionArgs = {
    verificationArgs: VerificationArgs.CreatorV1,
  };

  const { tx: unverifyTx } = await API.unverify(
    handler,
    authority,
    null,
    daItemManager.metadata,
    null,
    null,
    unverifyArgs,
  );

  await unverifyTx.assertSuccess(t);

  // Collection is set for item and verified.
  const metadataUnverified = await Metadata.fromAccountAddress(connection, daItemManager.metadata);
  spok(t, metadataUnverified.data, {
    creators: unverifiedCreators,
  });
});

test('Verify and Unverify: NFT member of NFT collection', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  // Create collection parent NFT.
  const collectionDetails: CollectionDetails = {
    __kind: 'V1',
    size: 0,
  };

  const daCollectionManager = await createAndMintDefaultCollectionParent(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
    collectionDetails,
  );

  // Create item NFT.
  const daItemManager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
    daCollectionManager.mint,
  );

  // Collection is set for item but unverified.
  const metadataInitial = await Metadata.fromAccountAddress(connection, daItemManager.metadata);
  spok(t, metadataInitial, {
    collection: { key: spokSamePubkey(daCollectionManager.mint), verified: false },
  });

  // Verify.
  const authority = payer;
  const verifyArgs: VerifyInstructionArgs = {
    verificationArgs: VerificationArgs.CollectionV1,
  };

  const { tx: verifyTx } = await API.verify(
    handler,
    authority,
    null,
    daItemManager.metadata,
    daCollectionManager.mint,
    daCollectionManager.metadata,
    daCollectionManager.masterEdition,
    verifyArgs,
  );

  await verifyTx.assertSuccess(t);

  // Collection is set for item and verified.
  const metadataVerified = await Metadata.fromAccountAddress(connection, daItemManager.metadata);
  spok(t, metadataVerified, {
    collection: { key: spokSamePubkey(daCollectionManager.mint), verified: true },
  });

  // Unverify.
  const unverifyArgs: UnverifyInstructionArgs = {
    verificationArgs: VerificationArgs.CollectionV1,
  };

  const { tx: unverifyTx } = await API.unverify(
    handler,
    authority,
    null,
    daItemManager.metadata,
    daCollectionManager.mint,
    daCollectionManager.metadata,
    unverifyArgs,
  );

  await unverifyTx.assertSuccess(t);

  // Collection is set for item and verified.
  const metadataUnverified = await Metadata.fromAccountAddress(connection, daItemManager.metadata);
  spok(t, metadataUnverified, {
    collection: { key: spokSamePubkey(daCollectionManager.mint), verified: false },
  });
});

test('Verify and Unverify: pNFT member of pNFT collection', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  // Create collection parent NFT.
  const collectionDetails: CollectionDetails = {
    __kind: 'V1',
    size: 0,
  };

  const daCollectionManager = await createAndMintDefaultCollectionParent(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    collectionDetails,
  );

  // Create item NFT.
  const daItemManager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
    daCollectionManager.mint,
  );

  // Collection is set for item but unverified.
  const metadataInitial = await Metadata.fromAccountAddress(connection, daItemManager.metadata);
  spok(t, metadataInitial, {
    collection: { key: spokSamePubkey(daCollectionManager.mint), verified: false },
  });

  // Verify.
  const authority = payer;
  const verifyArgs: VerifyInstructionArgs = {
    verificationArgs: VerificationArgs.CollectionV1,
  };

  const { tx: verifyTx } = await API.verify(
    handler,
    authority,
    null,
    daItemManager.metadata,
    daCollectionManager.mint,
    daCollectionManager.metadata,
    daCollectionManager.masterEdition,
    verifyArgs,
  );

  await verifyTx.assertSuccess(t);

  // Collection is set for item and verified.
  const metadataVerified = await Metadata.fromAccountAddress(connection, daItemManager.metadata);
  spok(t, metadataVerified, {
    collection: { key: spokSamePubkey(daCollectionManager.mint), verified: true },
  });

  // Unverify.
  const unverifyArgs: UnverifyInstructionArgs = {
    verificationArgs: VerificationArgs.CollectionV1,
  };

  const { tx: unverifyTx } = await API.unverify(
    handler,
    authority,
    null,
    daItemManager.metadata,
    daCollectionManager.mint,
    daCollectionManager.metadata,
    unverifyArgs,
  );

  await unverifyTx.assertSuccess(t);

  // Collection is set for item and verified.
  const metadataUnverified = await Metadata.fromAccountAddress(connection, daItemManager.metadata);
  spok(t, metadataUnverified, {
    collection: { key: spokSamePubkey(daCollectionManager.mint), verified: false },
  });
});
