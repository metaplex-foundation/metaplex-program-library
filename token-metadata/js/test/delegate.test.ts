import { DelegateArgs, TokenStandard } from '../src/generated';
import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { createAndMintDefaultAsset } from './utils/DigitalAssetManager';

killStuckProcess();

test('Delegate: create CollectionDelegate', async (t) => {
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
