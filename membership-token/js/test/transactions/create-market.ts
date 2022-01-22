import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

import { createCreateMarketInstruction } from '../../src/instructions';
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
  treasuryOwnerBump,
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
  treasuryOwnerBump: number;
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
      treasuryOwnerBump,
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
