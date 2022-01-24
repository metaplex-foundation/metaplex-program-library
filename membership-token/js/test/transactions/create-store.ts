import { Connection, Keypair, Transaction } from '@solana/web3.js';

import { createCreateStoreInstruction } from '../../src/mpl-membership-token';
import { createAndSignTransaction } from '../utils';

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

  const transaction = await createAndSignTransaction(instruction, connection, payer, [store]);

  return {
    store,
    transaction,
  };
};
