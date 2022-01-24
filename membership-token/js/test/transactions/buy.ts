import { Connection, PublicKey, Transaction, SYSVAR_CLOCK_PUBKEY } from '@solana/web3.js';
import { MetadataProgram } from '@metaplex-foundation/mpl-token-metadata';

import { createBuyInstruction } from '../../src/instructions';

interface BuyMembershipTokenParams {
  connection: Connection;
  buyer: PublicKey;
  buyerTokenAccount: PublicKey;
  sellingResource: PublicKey;
  tradeHistory: PublicKey;
  tradeHistoryBump: number;
  market: PublicKey;
  marketTreasuryHolder: PublicKey;
  vault: PublicKey;
  treasuryOwner: PublicKey;
  vaultOwnerBump: number;
  newMint: PublicKey;
  newMintEdition: PublicKey;
  newMintEditionMarker: PublicKey;
  newMintMetadata: PublicKey;
  newMintMasterEdition: PublicKey;
  newMintMasterMetadata: PublicKey;
}

export const createBuyTransaction = async ({
  connection,
  buyer,
  buyerTokenAccount,
  sellingResource,
  tradeHistory,
  tradeHistoryBump,
  market,
  marketTreasuryHolder,
  vault,
  treasuryOwner,
  vaultOwnerBump,
  newMint,
  newMintEdition,
  newMintEditionMarker,
  newMintMetadata,
  newMintMasterEdition,
  newMintMasterMetadata,
}: BuyMembershipTokenParams) => {
  const instruction = await createBuyInstruction(
    {
      // buyer wallet
      userWallet: buyer,
      // user token account
      userTokenAccount: buyerTokenAccount,
      // account which holds selling entities
      sellingResource,
      // token account for selling resource
      vault,
      // owner of selling resource token account PDA
      owner: treasuryOwner,
      // market account
      market,
      // PDA which creates on market for each buyer
      tradeHistory,
      // market treasury holder (buyer will send tokens to this account)
      treasuryHolder: marketTreasuryHolder,
      // newly generated mint address
      newMint: newMint,
      // newly generated mint metadata PDA
      newMetadata: newMintMetadata,
      // newly generated mint edition PDA
      newEdition: newMintEdition,
      // PDA which will be taken by newMint + newEdition
      editionMarker: newMintEditionMarker,
      // master edition for newly generated mint (genesis mint)
      masterEdition: newMintMasterEdition,
      // master edition metadata PDA (genesis mint metadata)
      masterEditionMetadata: newMintMasterMetadata,
      // solana system account
      clock: SYSVAR_CLOCK_PUBKEY,
      // metaplex token metadata program address
      tokenMetadataProgram: MetadataProgram.PUBKEY,
    },
    { tradeHistoryBump, vaultOwnerBump },
  );

  const tx = new Transaction();
  tx.add(instruction);
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  tx.feePayer = buyer;

  return { tx };
};
