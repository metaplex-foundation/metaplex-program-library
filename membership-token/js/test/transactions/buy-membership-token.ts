import { Connection, PublicKey, Transaction } from '@solana/web3.js';

import { createBuyInstruction } from '../../src/mpl-membership-token';

interface BuyMembershipTokenParams {
  connection: Connection;
  payer: PublicKey;
  sellingResource: PublicKey;
  vault: PublicKey;
  vaultOwner: PublicKey;
  vaultOwnerBump: number;
  resourceMint: PublicKey;
  resourceMintMasterEdition: PublicKey;
}

export const buyMembershipToken = async ({
  connection,
  payer,
  sellingResource,
  vault,
  vaultOwner,
  vaultOwnerBump,
  resourceMint,
  resourceMintMasterEdition,
}: BuyMembershipTokenParams) => {
  const instruction = await createBuyInstruction(
    {
      market: undefined,
      sellingResource,
      userTokenAccount: undefined,
      userWallet: undefined,
      tradeHistory: undefined,
      treasuryHolder: undefined,
      newMetadata: undefined,
      newEdition: undefined,
      masterEdition: resourceMintMasterEdition,
      newMint: resourceMint,
      editionMarker: undefined,
      vault,
      owner: vaultOwner,
      masterEditionMetadata: undefined,
      clock: undefined,
      tokenMetadataProgram: undefined,
    },
    { tradeHistoryBump: undefined, vaultOwnerBump },
  );

  const transaction = new Transaction();
  transaction.add(instruction);
  transaction.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  transaction.feePayer = payer;

  return { tx: transaction };
};
