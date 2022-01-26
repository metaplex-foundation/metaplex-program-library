import { SetStore } from '../src/transactions';
import { Store } from '../src/accounts/Store';
import { FEE_PAYER, NETWORK } from './utils';
import { Connection, PublicKey } from '@solana/web3.js';
import { NodeWallet } from './wallet';
import { airdrop } from '@metaplex-foundation/amman';

describe('Store', () => {
  let connection: Connection;
  jest.setTimeout(80000);

  beforeAll(async () => {
    connection = new Connection(NETWORK);
    await airdrop(connection, FEE_PAYER.publicKey, 2);
  });

  describe('Store', () => {
    test('SetStore', async () => {
      const wallet = new NodeWallet(FEE_PAYER);
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
      expect(typeof txId).toBe('string');
    });
  });
});
