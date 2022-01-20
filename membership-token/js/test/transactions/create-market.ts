import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

import { createCreateMarketInstruction } from '../../src/mpl-membership-token';
import { createAndSignTransaction } from '../utils';

export const createMarketTransaction = async ({
  store,
  payer,
  connection,
  sellingResourceOwner,
  sellingResource,
  mint,
  treasuryHolder,
  owner,
  treasyryOwnerBump,
  name,
  description,
  mutable,
  price,
  piecesInOneWallet,
  startDate,
  endDate,
}: {
  payer: Keypair;
  connection: Connection;
  store: Keypair;
  sellingResourceOwner: Keypair;
  sellingResource: PublicKey;
  mint: PublicKey;
  treasuryHolder: PublicKey;
  owner: PublicKey;
  treasyryOwnerBump: number;
  name: string;
  description: string;
  mutable: boolean;
  price: beet.bignum;
  piecesInOneWallet: beet.COption<beet.bignum>;
  startDate: beet.bignum;
  endDate: beet.COption<beet.bignum>;
}): Promise<{ market: Keypair; marketTx: Transaction }> => {
  const market = Keypair.generate();

  const instruction = createCreateMarketInstruction(
    {
      market: market.publicKey,
      store: store.publicKey,
      sellingResourceOwner: sellingResourceOwner.publicKey,
      sellingResource,
      mint,
      treasuryHolder,
      owner,
    },
    {
      name,
      description,
      treasyryOwnerBump,
      mutable,
      price,
      piecesInOneWallet,
      startDate,
      endDate,
    },
  );

  const marketTx: Transaction = await createAndSignTransaction(instruction, connection, payer, [
    market,
    sellingResourceOwner,
  ]);
  return {
    market,
    marketTx,
  };
};
