import {
  DelegateArgs,
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
      manager.mint.toBuffer(),
      Buffer.from('transfer_delegate'),
      delegate.toBuffer(),
      payer.publicKey.toBuffer(),
    ],
    PROGRAM_ID,
  );
  amman.addr.addLabel('Delegate Record', delegateRecord);

  const delegateArgs: DelegateArgs = {
    __kind: 'TransferV1',
    amount: 1,
  };

  const { tx: delegateTx } = await API.delegate(
    t,
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
    delegateState: {
      delegate: spokSamePubkey(delegate),
      role: DelegateRole.Transfer,
      hasData: false,
    },
  });

  // revoke

  const { tx: revoketeTx } = await API.revoke(
    t,
    delegateRecord,
    delegate,
    manager.mint,
    manager.metadata,
    manager.masterEdition,
    payer.publicKey,
    payer,
    RevokeArgs.TransferV1,
    handler,
    manager.token,
  );

  await revoketeTx.assertSuccess(t);

  metadata = await Metadata.fromAccountAddress(connection, manager.metadata);

  spok(t, metadata, {
    delegateState: null,
  });
});
