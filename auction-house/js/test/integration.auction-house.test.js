import test from 'tape';
import { SetStore } from '../src/transactions';
import { Store } from '../src/accounts/Store';
import { connectionURL } from './utils';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { airdrop, PayerTransactionHandler } from '@metaplex-foundation/amman';

test('test_action', async (t) => {
  const payer = Keypair.generate();
  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);
  await airdrop(connection, payer.publicKey, 2);

  t.ok(True, 'Test runs');
});
