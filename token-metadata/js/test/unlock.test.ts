import { getAccount } from '@solana/spl-token';
import { BN } from 'bn.js';
import spok from 'spok';
import { AssetState, DelegateArgs, Metadata, PROGRAM_ID, TokenStandard } from '../src/generated';
import test from 'tape';
import { amman, InitTransactions, killStuckProcess } from './setup';
import { spokSameBigint } from './utils';
import { createAndMintDefaultAsset } from './utils/digital-asset-manager';
import { PublicKey } from '@solana/web3.js';

killStuckProcess();

test('Unlock: unlock NonFungible asset', async (t) => {
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

  let metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Unlocked /* asset should be unlocked */,
  });

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: false,
    });
  }

  // lock asset

  const { tx: lockTx } = await API.lock(
    payer,
    manager.mint,
    manager.metadata,
    payer,
    handler,
    null,
    manager.token,
    manager.masterEdition,
  );
  await lockTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Locked /* asset should be locked */,
  });

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
    payer,
    handler,
    null,
    manager.token,
    manager.masterEdition,
  );
  await unlockTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Unlocked /* asset should be unlocked */,
  });

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: false,
    });
  }
});

test('Unlock: unlock ProgrammableNonFungible asset', async (t) => {
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

  let metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Unlocked /* asset should be unlocked */,
  });

  // lock asset

  const { tx: lockTx } = await API.lock(
    payer,
    manager.mint,
    manager.metadata,
    payer,
    handler,
    null,
    manager.token,
    manager.masterEdition,
  );
  await lockTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Locked /* asset should be locked */,
  });

  // unlock asset

  const { tx: unlockTx } = await API.unlock(
    payer,
    manager.mint,
    manager.metadata,
    payer,
    handler,
    null,
    manager.token,
    manager.masterEdition,
  );
  await unlockTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Unlocked /* asset should be unlocked */,
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

  let metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Unlocked /* asset should be unlocked */,
  });

  // lock asset

  const { tx: lockTx } = await API.lock(
    payer,
    manager.mint,
    manager.metadata,
    payer,
    handler,
    null,
    manager.token,
    manager.masterEdition,
  );
  await lockTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Locked /* asset should be locked */,
  });

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: true,
    });
  }

  // lock asset

  const { tx: unlockTx } = await API.unlock(
    payer,
    manager.mint,
    manager.metadata,
    payer,
    handler,
    null,
    manager.token,
    manager.masterEdition,
  );
  await unlockTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Unlocked /* asset should be unlocked */,
  });

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

  let metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Unlocked /* asset should be unlocked */,
  });

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');
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

  const args: DelegateArgs = {
    __kind: 'UtilityV1',
    amount: 1,
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
    args,
    handler,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // lock asset with delegate

  const { tx: utilityTx } = await API.lock(
    delegate,
    manager.mint,
    manager.metadata,
    payer,
    handler,
    delegateRecord,
    manager.token,
    manager.masterEdition,
  );
  await utilityTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Locked /* asset should be locked */,
  });

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
    payer,
    handler,
    delegateRecord,
    manager.token,
    manager.masterEdition,
  );
  await unlockTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Unlocked /* asset should be unlocked */,
  });

  if (manager.token) {
    const tokenAccount = await getAccount(connection, manager.token);

    spok(t, tokenAccount, {
      isFrozen: false,
    });
  }
});

test.only('Unlock: wrong delegate unlock NonFungible asset', async (t) => {
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

  let metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Unlocked /* asset should be unlocked */,
  });

  // creates a delegate

  const [, delegate] = await API.getKeypair('Delegate');
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

  const args: DelegateArgs = {
    __kind: 'UtilityV1',
    amount: 1,
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
    args,
    handler,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // lock asset with delegate

  const { tx: utilityTx } = await API.lock(
    delegate,
    manager.mint,
    manager.metadata,
    payer,
    handler,
    delegateRecord,
    manager.token,
    manager.masterEdition,
  );
  await utilityTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    assetState: AssetState.Locked /* asset should be locked */,
  });

  // creates a transfer delegate

  const [, transferDelegate] = await API.getKeypair('Delegate');

  const argsTransfer: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: transferDelegateTx } = await API.delegate(
    delegateRecord,
    transferDelegate.publicKey,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    payer.publicKey,
    payer,
    argsTransfer,
    handler,
    manager.token,
  );

  await transferDelegateTx.assertSuccess(t);

  // unlock asset with delegate

  const { tx: unlockTx } = await API.unlock(
    transferDelegate,
    manager.mint,
    manager.metadata,
    payer,
    handler,
    delegateRecord,
    manager.token,
    manager.masterEdition,
  );
  await unlockTx.assertError(t, /Invalid authority type/);
});
