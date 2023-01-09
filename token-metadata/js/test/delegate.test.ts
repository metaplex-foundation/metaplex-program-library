import {
  DelegateArgs,
  DelegateRecord,
  DelegateRole,
  Metadata,
  PROGRAM_ID,
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

test('Delegate: create collection delegate', async (t) => {
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
      Buffer.from('collection_delegate'),
      payer.publicKey.toBuffer(),
      delegate.toBuffer(),
    ],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Delegate Record', delegateRecord);

  const args: DelegateArgs = {
    __kind: 'CollectionV1',
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
    args,
    handler,
  );

  await delegateTx.assertSuccess(t);
});

test('Delegate: create sale delegate', async (t) => {
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

  const args: DelegateArgs = {
    __kind: 'SaleV1',
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
    args,
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

  const pda = await DelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    role: DelegateRole.Sale,
  });
});

test('Delegate: create transfer delegate', async (t) => {
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

  const args: DelegateArgs = {
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
    args,
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

  const pda = await DelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    role: DelegateRole.Transfer,
  });
});

test('Delegate: fail to create sale delegate on NFT', async (t) => {
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

  const args: DelegateArgs = {
    __kind: 'SaleV1',
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
    args,
    handler,
    manager.token,
  );

  await delegateTx.assertError(t, /Invalid token standard/);
});

test('Delegate: replace transfer delegate', async (t) => {
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

  const args: DelegateArgs = {
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
    args,
    handler,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  let tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  let pda = await DelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    role: DelegateRole.Transfer,
  });

  // creates a new delegate

  const [newDelegate] = await API.getKeypair('Delegate');

  const { tx: delegateTx2 } = await API.delegate(
    delegateRecord,
    newDelegate,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    payer.publicKey,
    payer,
    args,
    handler,
    manager.token,
  );

  await delegateTx2.assertSuccess(t);

  // asserts

  tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(newDelegate),
  });

  pda = await DelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(newDelegate),
    role: DelegateRole.Transfer,
  });
});

test('Delegate: create utility delegate', async (t) => {
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

  const args: DelegateArgs = {
    __kind: 'UtilityV1',
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
    args,
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

  const pda = await DelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    role: DelegateRole.Utility,
  });

  const metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    persistentDelegate: DelegateRole.Utility,
  });
});

test('Delegate: try replace sale delegate', async (t) => {
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

  const args: DelegateArgs = {
    __kind: 'SaleV1',
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
    args,
    handler,
    manager.token,
  );

  await delegateTx.assertSuccess(t);

  // asserts

  let tokenAccount = await getAccount(connection, manager.token);

  spok(t, tokenAccount, {
    delegatedAmount: spokSameBigint(new BN(1)),
    delegate: spokSamePubkey(delegate),
  });

  let pda = await DelegateRecord.fromAccountAddress(connection, delegateRecord);

  spok(t, pda, {
    delegate: spokSamePubkey(delegate),
    role: DelegateRole.Sale,
  });

  // creates a transfer delegate

  const [newDelegate] = await API.getKeypair('Delegate');

  const args2: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
    authorizationData: null,
  };

  const { tx: delegateTx2 } = await API.delegate(
    delegateRecord,
    newDelegate,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    payer.publicKey,
    payer,
    args2,
    handler,
    manager.token,
  );

  await delegateTx2.assertError(t, /Delegate already exists/);
});
