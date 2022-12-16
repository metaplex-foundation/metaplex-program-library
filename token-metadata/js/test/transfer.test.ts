import test from 'tape';
import { InitTransactions, killStuckProcess } from './setup';
import { Keypair } from '@solana/web3.js';
import { createAndMintDefaultAsset } from './utils/DigitalAssetManager';
import { createAssociatedTokenAccount } from '@solana/spl-token';

killStuckProcess();

test('Transfer: NonFungible asset', async (t) => {
  const API = new InitTransactions();
  const { fstTxHandler: handler, payerPair: payer, connection } = await API.payer();

  const { mint, metadata, masterEdition, token } = await createAndMintDefaultAsset(
    t,
    API,
    handler,
    payer,
  );

  const owner = payer;
  const destination = Keypair.generate();
  const destinationToken = await createAssociatedTokenAccount(
    connection,
    payer,
    mint,
    destination.publicKey,
  );
  const amount = 1;

  const { tx: transferTx } = await API.transfer(
    owner,
    token,
    mint,
    metadata,
    masterEdition,
    destination.publicKey,
    destinationToken,
    amount,
    handler,
  );

  await transferTx.assertSuccess(t);
});
