import test from 'tape';
import { SetStore } from '../src/transactions';
import { Store } from '../src/accounts/Store';
import { connectionURL } from './utils';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { NodeWallet } from './wallet';
import { airdrop } from '@metaplex-foundation/amman';

test('set-store', async (t) => {
  const payer = Keypair.generate();
  const connection = new Connection(connectionURL, 'confirmed');
  await airdrop(connection, payer.publicKey, 2);

  const wallet = new NodeWallet(payer);
  const storeId = await Store.getPDA(wallet.publicKey);
  let tx = new SetStore(
    { feePayer: wallet.publicKey },
    {
      admin: new PublicKey(wallet.publicKey),
      store: storeId,
      isPublic: true,
    },
  );
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;

  tx = await wallet.signTransaction(tx);
  const txId = await connection.sendRawTransaction(tx.serialize(), {});
  t.ok(txId, 'a txId should be returned');
});
