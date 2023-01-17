import { getAccount } from '@solana/spl-token';
import { BN } from 'bn.js';
import spok from 'spok';
import { DelegateArgs, TokenRecord, TokenStandard, TokenState } from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { spokSameBigint } from './utils';
import { createAndMintDefaultAsset } from './utils/digital-asset-manager';
import { findTokenRecordPda } from './utils/programmable';

killStuckProcess();

test('Lock: owner lock NonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
  );

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(1)),
      isFrozen: false,
      owner: payer.publicKey,
    });
  }

  // lock asset

  const { tx: lockTx } = await API.lock(
    payer,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    null,
    null,
    manager.masterEdition,
  );
  await lockTx.assertError(t, /Invalid authority type/);
});

test('Lock: owner lock ProgrammableNonFungible asset', async (t) => {
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

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(1)),
      isFrozen: true,
      owner: payer.publicKey,
    });
  }

  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, payer.publicKey);
  amman.addr.addLabel('Token Record', tokenRecord);

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Unlocked /* asset should be unlocked */,
  });

  // lock asset

  const { tx: lockTx } = await API.lock(
    payer,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    tokenRecord,
  );
  await lockTx.assertError(t, /Invalid authority type/);
});

test('Lock: delegate lock ProgrammableNonFungible asset', async (t) => {
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

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(1)),
      isFrozen: true,
      owner: payer.publicKey,
    });
  }

  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, payer.publicKey);
  amman.addr.addLabel('Token Record', tokenRecord);

  let pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

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
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // lock asset with delegate

  const { tx: lockTx } = await API.lock(
    delegate,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    tokenRecord,
    null,
    manager.masterEdition,
  );
  await lockTx.assertSuccess(t);

  pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Locked /* asset should be locked */,
  });
});

test('Lock: delegate lock NonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
  );

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(1)),
      isFrozen: false,
      owner: payer.publicKey,
    });
  }

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');

  const args: DelegateArgs = {
    __kind: 'UtilityV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // lock asset with delegate

  const { tx: lockTx } = await API.lock(
    delegate,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    null,
    null,
    manager.masterEdition,
  );
  await lockTx.assertSuccess(t);

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: true,
    });
  }
});

test('Lock: lock Fungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.Fungible,
    null,
    100,
  );

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(100)),
      isFrozen: false,
      owner: payer.publicKey,
    });
  }

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');

  const args: DelegateArgs = {
    __kind: 'UtilityV1',
    amount: 100,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    null,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // lock asset

  const { tx: lockTx } = await API.lock(
    delegate,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    null,
    payer.publicKey,
  );
  await lockTx.assertSuccess(t);

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: true,
    });
  }
});

test('Lock: lock ProgrammableNonFungible asset with wrong authority', async (t) => {
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

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(1)),
      isFrozen: true,
      owner: payer.publicKey,
    });
  }

  // token record PDA
  const tokenRecord = findTokenRecordPda(manager.mint, payer.publicKey);
  amman.addr.addLabel('Token Record', tokenRecord);

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Unlocked /* asset should be unlocked */,
  });

  // lock asset

  const [, wrongApprover] = await amman.genLabeledKeypair('Wrong Approver');

  const { tx: lockTx } = await API.lock(
    wrongApprover,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
  );
  await lockTx.assertError(t, /Invalid authority type/);
});

test('Lock: wrong delegate lock NonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const manager = await createAndMintDefaultAsset(
    t,
    connection,
    API,
    handler,
    payer,
    TokenStandard.NonFungible,
  );

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(1)),
      isFrozen: false,
      owner: payer.publicKey,
    });
  }

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');

  const args: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // lock asset with wrong delegate

  const [, wrongDelegate] = await API.getKeypair('Wrong Delegate');

  const { tx: lockTx } = await API.lock(
    wrongDelegate,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    null,
    manager.masterEdition,
  );
  await lockTx.assertError(t, /not been delegated to this user/);
});

test('Lock: wrong delegate lock ProgrammableNonFungible asset', async (t) => {
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

  const tokenRecord = findTokenRecordPda(manager.mint, payer.publicKey);
  amman.addr.addLabel('Token Record', tokenRecord);

  const pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Unlocked /* asset should be unlocked */,
  });

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(1)),
      isFrozen: true,
      owner: payer.publicKey,
    });
  }

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');

  const args: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // lock asset with delegate

  const { tx: utilityTx } = await API.lock(
    delegate,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    tokenRecord,
    null,
    manager.masterEdition,
  );
  await utilityTx.assertError(t, /Invalid authority type/);
});

test('Lock: already locked ProgrammableNonFungible asset', async (t) => {
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

  const tokenRecord = findTokenRecordPda(manager.mint, payer.publicKey);
  amman.addr.addLabel('Token Record', tokenRecord);

  let pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Unlocked /* asset should be unlocked */,
  });

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(1)),
      isFrozen: true,
      owner: payer.publicKey,
    });
  }

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');

  const args: DelegateArgs = {
    __kind: 'UtilityV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx } = await API.delegate(
    delegate.publicKey,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await delegateTx.assertSuccess(t);

  // lock asset with delegate

  const { tx: utilityTx } = await API.lock(
    delegate,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    tokenRecord,
    null,
    manager.masterEdition,
  );
  await utilityTx.assertSuccess(t);

  pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Locked /* asset should be unlocked */,
  });

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(1)),
      isFrozen: true,
      owner: payer.publicKey,
    });
  }

  // tries to create a new delegate

  const [, newDelegate] = await API.getKeypair('Delegate');

  const { tx: newDelegateTx } = await API.delegate(
    newDelegate.publicKey,
    manager.mint,
    manager.metadata,
    payer.publicKey,
    payer,
    args,
    handler,
    null,
    manager.masterEdition,
    manager.token,
    tokenRecord,
  );

  await newDelegateTx.assertError(t, /Token is locked/);
});
