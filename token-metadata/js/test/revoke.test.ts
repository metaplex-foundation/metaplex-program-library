import {
  DelegateArgs,
  DelegateRecord,
  DelegateRole,
  Metadata,
  PROGRAM_ID,
  RevokeArgs,
  TokenStandard,
} from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { createAndMintDefaultAsset } from './utils/DigitalAssetManager';
import spok from 'spok';
import { spokSameBigint, spokSamePubkey } from './utils';
import { BN } from 'bn.js';
import { getAccount } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';

killStuckProcess();

test('Revoke: revoke transfer delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [delegate] = await API.getKeypair('Delegate');
  // delegate PDA
  const [delegateRecord] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('metadata'),
      PROGRAM_ID.toBuffer(),
      manager.mint.toBuffer(),
      Buffer.from('persistent_delegate'),
      payer.publicKey.toBuffer(),
    ],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Delegate Record', delegateRecord);

  const delegateArgs: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegateRecord,
    delegate,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    payer.publicKey,
    payer,
    delegateArgs,
    handler,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  let metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    persistentDelegate: DelegateRole.Transfer,
  });

  // revoke

  const { tx: revoketeTx } = await API.revoke(
    delegateRecord,
    delegate,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    payer,
    payer,
    RevokeArgs.TransferV1,
    handler,
    manager.token,
  );

  await revoketeTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    persistentDelegate: null,
  });
});

test('Revoke: revoke collection delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');
  // delegate PDA
  const [delegateRecord] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('metadata'),
      PROGRAM_ID.toBuffer(),
      manager.mint.toBuffer(),
      Buffer.from('collection_delegate'),
      payer.publicKey.toBuffer(),
      delegate.publicKey.toBuffer(),
    ],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Delegate Record', delegateRecord);

  const delegateArgs: DelegateArgs = {
    __kind: 'CollectionV1',
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegateRecord,
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    payer.publicKey,
    payer,
    delegateArgs,
    handler,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const account = await DelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, account, {
    delegate: spokSamePubkey(delegate.publicKey),
    role: DelegateRole.Collection,
  });

  // revoke

  const { tx: revoketeTx } = await API.revoke(
    delegateRecord,
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    payer,
    payer,
    RevokeArgs.CollectionV1,
    handler,
    manager.token,
  );

  await revoketeTx.assertSuccess(t);

  try {
    await DelegateRecord.fromAccountAddress(connection, delegateRecord);
    t.fail(`Delegate account ${delegateRecord} was found`);
  } catch (err) {
    // we are expecting an error, since the account must be deleted
  }
});

test('Revoke: self-revoke collection delegate', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.ProgrammableNonFungible,
  );

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');
  // delegate PDA
  const [delegateRecord] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('metadata'),
      PROGRAM_ID.toBuffer(),
      manager.mint.toBuffer(),
      Buffer.from('collection_delegate'),
      payer.publicKey.toBuffer(),
      delegate.publicKey.toBuffer(),
    ],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Delegate Record', delegateRecord);

  const delegateArgs: DelegateArgs = {
    __kind: 'CollectionV1',
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegateRecord,
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    payer.publicKey,
    payer,
    delegateArgs,
    handler,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const account = await DelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, account, {
    delegate: spokSamePubkey(delegate.publicKey),
    role: DelegateRole.Collection,
  });

  // revoke

  const { tx: revoketeTx } = await API.revoke(
    delegateRecord,
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    delegate,
    payer,
    RevokeArgs.CollectionV1,
    handler,
    manager.token,
  );

  await revoketeTx.assertSuccess(t);

  try {
    await DelegateRecord.fromAccountAddress(connection, delegateRecord);
    t.fail(`Delegate account ${delegateRecord} was found`);
  } catch (err) {
    // we are expecting an error, since the account must be deleted
  }

  // try to revoke again

  const { tx: revoketeTx2 } = await API.revoke(
    delegateRecord,
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    delegate,
    payer,
    RevokeArgs.CollectionV1,
    handler,
    manager.token,
  );

  await revoketeTx2.assertError(t, /Uninitialized/);
});
