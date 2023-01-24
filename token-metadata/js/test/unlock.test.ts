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

test('Unlock: owner unlock NonFungible asset', async (t) => {
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

  // lock asset

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

  // unlock asset

  const { tx: unlockTx } = await API.unlock(
    payer,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    null,
    manager.masterEdition,
  );
  await unlockTx.assertError(t, /Invalid authority type/);

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: true,
    });
  }
});

test('Unlock: owner unlock ProgrammableNonFungible asset', async (t) => {
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

  // lock asset

  const { tx: lockTx } = await API.lock(
    delegate,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    tokenRecord,
    manager.masterEdition,
  );
  await lockTx.assertSuccess(t);

  pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Locked /* asset should be locked */,
  });

  // unlock asset

  const { tx: unlockTx } = await API.unlock(
    payer,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    tokenRecord,
    manager.masterEdition,
  );
  await unlockTx.assertError(t, /Invalid authority type/);

  pda = await TokenRecord.fromAccountAddress(connection, tokenRecord);

  spok(t, pda, {
    state: TokenState.Locked /* should be unlocked still */,
  });

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: true,
    });
  }
});

test('Unlock: unlock Fungible asset', async (t) => {
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
    __kind: 'StandardV1',
    amount: 100,
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

  // lock asset

  const { tx: unlockTx } = await API.unlock(
    delegate,
    manager.mint,
    manager.metadata,
    manager.token,
    payer,
    handler,
    null,
    payer.publicKey,
  );
  await unlockTx.assertSuccess(t);

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: false,
    });
  }
});

test('Unlock: delegate unlock NonFungible asset', async (t) => {
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

  const { tx: utilityTx } = await API.lock(
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
  await utilityTx.assertSuccess(t);

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: true,
    });
  }

  // unlock asset with delegate

  const { tx: unlockTx } = await API.unlock(
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
  await unlockTx.assertSuccess(t);

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: false,
    });
  }
});
