import {
  DelegateArgs,
  DelegateRecord,
  DelegateRole,
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
