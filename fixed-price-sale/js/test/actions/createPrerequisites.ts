import { Connection, Keypair } from '@solana/web3.js';
import { Amman, PayerTransactionHandler } from '@metaplex-foundation/amman-client';

import { connectionURL } from '../utils';

export const createPrerequisites = async () => {
  const payer = Keypair.generate();

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await Amman.instance().airdrop(connection, payer.publicKey, 30);

  return { payer, connection, transactionHandler };
};
