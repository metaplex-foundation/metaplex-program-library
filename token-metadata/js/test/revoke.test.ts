import {
  DelegateArgs,
  MetadataDelegateRecord,
  PROGRAM_ID,
  RevokeArgs,
  TokenDelegateRole,
  TokenRecord,
  TokenStandard,
} from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { createAndMintDefaultAsset } from './utils/digital-asset-manager';
import spok from 'spok';
import { spokSameBigint, spokSamePubkey } from './utils';
import { BN } from 'bn.js';
import { getAccount } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { findTokenRecordPda } from './utils/programmable';

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
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const delegateArgs: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    delegateArgs,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  let pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    delegateRole: TokenDelegateRole.Transfer,
  });

  // revoke

  const { tx: revoketeTx } = await API.revoke(
    delegate,
    manager.mint,
    manager.metadata,
    payer,
    payer,
    RevokeArgs.TransferV1,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await revoketeTx.assertSuccess(t);

  pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: null,
    delegateRole: null,
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
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    delegateArgs,
    handler,
    delegateRecord,
    manager.masterEdition,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const account = await MetadataDelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, account, {
    delegate: spokSamePubkey(delegate.publicKey),
    mint: spokSamePubkey(manager.mint),
  });

  // revoke

  const { tx: revoketeTx } = await API.revoke(
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    payer,
    payer,
    RevokeArgs.CollectionV1,
    handler,
    delegateRecord,
    manager.masterEdition,
    manager.token,
  );

  await revoketeTx.assertSuccess(t);

  try {
    await MetadataDelegateRecord.fromAccountAddress(connection, delegateRecord);
    t.fail(`Metadata delegate account ${delegateRecord} was found`);
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
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    delegateArgs,
    handler,
    delegateRecord,
    manager.masterEdition,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const account = await MetadataDelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, account, {
    delegate: spokSamePubkey(delegate.publicKey),
    mint: spokSamePubkey(manager.mint),
  });

  // revoke

  const { tx: revoketeTx } = await API.revoke(
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    delegate,
    payer,
    RevokeArgs.CollectionV1,
    handler,
    delegateRecord,
    manager.masterEdition,
    manager.token,
  );

  await revoketeTx.assertSuccess(t);

  try {
    await MetadataDelegateRecord.fromAccountAddress(connection, delegateRecord);
    t.fail(`Delegate account ${delegateRecord} was found`);
  } catch (err) {
    // we are expecting an error, since the account must be deleted
  }

  // try to revoke again

  const { tx: revoketeTx2 } = await API.revoke(
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    delegate,
    payer,
    RevokeArgs.CollectionV1,
    handler,
    delegateRecord,
    manager.token,
    manager.masterEdition,
  );

  await revoketeTx2.assertError(t, /Delegate not found/);
});

test('Revoke: revoke locked transfer delegate', async (t) => {
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
  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  const delegateArgs: DelegateArgs = {
    __kind: 'LockedTransferV1',
    amount: 1,
    lockedAddress: PublicKey.default,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    delegateArgs,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  const tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  let pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    delegateRole: TokenDelegateRole.LockedTransfer,
    lockedTransfer: spokSamePubkey(PublicKey.default),
  });

  // revoke

  const { tx: revoketeTx } = await API.revoke(
    delegate,
    manager.mint,
    manager.metadata,
    payer,
    payer,
    RevokeArgs.LockedTransferV1,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await revoketeTx.assertSuccess(t);

  pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    delegate: null,
    delegateRole: null,
    lockedTransfer: null,
  });
});
