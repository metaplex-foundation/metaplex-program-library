import { Connection, Keypair, Transaction } from '@solana/web3.js';

import { createCreateStoreInstruction } from '../../src/mpl-membership-token';

export const createStoreTransaction = async (
  payer: Keypair,
  connection: Connection,
): Promise<{ store: Keypair; transaction: Transaction }> => {
  const store = Keypair.generate();

  const instruction = createCreateStoreInstruction(
    {
      store: store.publicKey,
      admin: payer.publicKey,
    },
    {
      name: 'Brand new Store',
      description: 'Description the Store',
    },
  );

  const transaction = new Transaction();
  transaction.add(instruction);
  transaction.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  transaction.feePayer = payer.publicKey;
  transaction.partialSign(store);

  return {
    store,
    transaction,
  };
};
