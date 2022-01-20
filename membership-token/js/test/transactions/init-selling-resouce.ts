import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import { bignum } from '@metaplex-foundation/beet';

import { createInitSellingResourceInstruction } from '../../src/mpl-membership-token';

export const createInitSellingResourceTransaction = async ({
  payer,
  connection,
  store,
  resourceMint,
  masterEdition,
  vault,
  owner,
  resourceToken,
  masterEditionBump,
  vaultOwnerBump,
  maxSupply,
}: {
  payer: Keypair;
  connection: Connection;
  store: Keypair;
  resourceMint: PublicKey;
  masterEdition: PublicKey;
  vault: Keypair;
  owner: PublicKey;
  resourceToken: PublicKey;
  masterEditionBump: number;
  vaultOwnerBump: number;
  maxSupply: bignum;
}): Promise<{ sellingResource: Keypair; initSellingResourceTx: Transaction }> => {
  const sellingResource = Keypair.generate();

  const instruction = createInitSellingResourceInstruction(
    {
      store: store.publicKey,
      admin: payer.publicKey,
      sellingResource: sellingResource.publicKey,
      sellingResourceOwner: payer.publicKey,
      masterEdition,
      resourceMint,
      resourceToken,
      vault: vault.publicKey,
      owner,
    },
    {
      masterEditionBump,
      vaultOwnerBump,
      maxSupply,
    },
  );

  const initSellingResourceTx = new Transaction();
  initSellingResourceTx.add(instruction);
  initSellingResourceTx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  initSellingResourceTx.feePayer = payer.publicKey;
  initSellingResourceTx.partialSign(sellingResource, vault);

  return {
    sellingResource,
    initSellingResourceTx,
  };
};
