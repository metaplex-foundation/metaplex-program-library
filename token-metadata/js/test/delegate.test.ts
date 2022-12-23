import { DelegateArgs, DelegateRole, Metadata, TokenStandard } from '../src/generated';
import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { createAndMintDefaultAsset } from './utils/DigitalAssetManager';
import spok from 'spok';
import { spokSameBigint, spokSamePubkey } from './utils';
import { BN } from 'bn.js';
import { getAccount } from '@solana/spl-token';

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

  const args: DelegateArgs = {
    __kind: 'CollectionV1',
  };

  const { tx: delegateTx } = await API.delegate(
    t,
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

  const args: DelegateArgs = {
    __kind: 'SaleV1',
    amount: 1,
  };

  const { tx: delegateTx, delegate } = await API.delegate(
    t,
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

  const metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    delegateState: {
      delegate: spokSamePubkey(delegate),
      role: DelegateRole.Sale,
      hasData: false,
    },
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

  const args: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
  };

  const { tx: delegateTx, delegate } = await API.delegate(
    t,
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

  const metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    delegateState: {
      delegate: spokSamePubkey(delegate),
      role: DelegateRole.Transfer,
      hasData: false,
    },
  });
});
