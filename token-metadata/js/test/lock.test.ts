import { getAccount } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { BN } from 'bn.js';
import spok from 'spok';
import {
  AssetData,
  DelegateArgs,
  TokenDelegateRole,
  TokenRecord,
  TokenStandard,
  TokenState,
} from '../src/generated';
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
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
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
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
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
    __kind: 'StandardV1',
    amount: 1,
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
  const { fstTxHandler: handler, authorityPair: authority, connection } = await API.authority();

  // initialize a mint account

  const { tx: splMintTx, mint } = await API.createMintAccount(authority, connection, handler);
  await splMintTx.assertSuccess(t);

  // create the metadata

  const data: AssetData = {
    name: 'Fungible',
    symbol: 'FUN',
    uri: 'uri',
    sellerFeeBasisPoints: 0,
    creators: [
      {
        address: authority.publicKey,
        share: 100,
        verified: false,
      },
    ],
    primarySaleHappened: false,
    isMutable: true,
    tokenStandard: TokenStandard.Fungible,
    collection: null,
    uses: null,
    collectionDetails: null,
    ruleSet: null,
  };

  const { tx: createTx, metadata } = await API.create(t, authority, data, 0, 0, handler, mint);
  // executes the transaction
  await createTx.assertSuccess(t);

  // initialize a token account

  const {
    fstTxHandler: payerHandler,
    payerPair: payer,
    connection: payerConnection,
  } = await API.payer();

  const { tx: tokenTx, token } = await API.createTokenAccount(
    mint,
    payer,
    payerConnection,
    payerHandler,
    payer.publicKey,
  );
  await tokenTx.assertSuccess(t);

  // mint

  const { tx: mintTx } = await API.mint(
    t,
    connection,
    authority,
    mint,
    metadata,
    null,
    null,
    100,
    handler,
    token,
  );
  await mintTx.assertSuccess(t);

  if (token) {
    const tokenAccount = await getAccount(connection, token);

    spok(t, tokenAccount, {
      amount: spokSameBigint(new BN(100)),
      isFrozen: false,
      owner: payer.publicKey,
    });
  }

  // lock asset

  const { tx: lockTx } = await API.lock(
    authority /* freeze authority */,
    mint,
    metadata,
    token,
    payer,
    handler,
    null,
    payer.publicKey,
  );
  await lockTx.assertSuccess(t);

  if (token) {
    const tokenAccount = await getAccount(connection, token);

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
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
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
    __kind: 'StandardV1',
    amount: 1,
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
  await lockTx.assertError(t, /Invalid authority type/);
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

  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
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

  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
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

test('Lock: LockedTransfer delegate lock ProgrammableNonFungible asset', async (t) => {
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
  const tokenRecord = findTokenRecordPda(manager.mint, manager.token);
  amman.addr.addLabel('Token Record', tokenRecord);

  let pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Unlocked /* asset should be unlocked */,
  });

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');

  const args: DelegateArgs = {
    __kind: 'LockedTransferV1',
    amount: 1,
    lockedAddress: PublicKey.default,
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
    delegateRole: TokenDelegateRole.LockedTransfer,
    lockedTransfer: PublicKey.default,
  });
});
