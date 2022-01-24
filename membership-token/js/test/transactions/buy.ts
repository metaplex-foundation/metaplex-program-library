import { Connection, PublicKey, Transaction, SYSVAR_CLOCK_PUBKEY } from '@solana/web3.js';
import { MetadataProgram } from '@metaplex-foundation/mpl-token-metadata';

import { createBuyInstruction } from '../../src/instructions';

interface BuyMembershipTokenParams {
  connection: Connection;
  buyer: PublicKey;
  userTokenAccount: PublicKey;
  resourceMintEditionMarker: PublicKey;
  resourceMintMasterEdition: PublicKey;
  resourceMintMasterMetadata: PublicKey;
  sellingResource: PublicKey;
  tradeHistory: PublicKey;
  tradeHistoryBump: number;
  market: PublicKey;
  marketTreasuryHolder: PublicKey;
  treasuryOwner: PublicKey;
  vault: PublicKey;
  vaultOwnerBump: number;
  newMint: PublicKey;
  newMintEdition: PublicKey;
  newMintMetadata: PublicKey;
}

export const createBuyTransaction = async ({
  connection,
  buyer,
  userTokenAccount,
  resourceMintEditionMarker,
  resourceMintMasterEdition,
  resourceMintMasterMetadata,
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
  newMintMetadata,
}: BuyMembershipTokenParams) => {
  const instruction = await createBuyInstruction(
    {
      // buyer wallet
      userWallet: buyer,
      // user token account
      userTokenAccount,
      // resource mint edition marker PDA
      editionMarker: resourceMintEditionMarker,
      // resource mint master edition
      masterEdition: resourceMintMasterEdition,
      // resource mint master edition metadata PDA
      masterEditionMetadata: resourceMintMasterMetadata,
      // token account for selling resource
      vault,
      // account which holds selling entities
      sellingResource,
      // owner of selling resource token account PDA
      owner: treasuryOwner,
      // market account
      market,
      // PDA which creates on market for each buyer
      tradeHistory,
      // market treasury holder (buyer will send tokens to this account)
      treasuryHolder: marketTreasuryHolder,
      // newly generated mint address
      newMint,
      // newly generated mint metadata PDA
      newMetadata: newMintMetadata,
      // newly generated mint edition PDA
      newEdition: newMintEdition,
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
