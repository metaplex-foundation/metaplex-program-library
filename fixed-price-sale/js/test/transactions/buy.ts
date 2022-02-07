import { Connection, PublicKey, Transaction, SYSVAR_CLOCK_PUBKEY } from '@solana/web3.js';
import { MetadataProgram } from '@metaplex-foundation/mpl-token-metadata';

import { createBuyInstruction } from '../../src/instructions';

interface BuyParams {
  connection: Connection;
  buyer: PublicKey;
  userTokenAccount: PublicKey;
  resourceMintMetadata: PublicKey;
  resourceMintEditionMarker: PublicKey;
  resourceMintMasterEdition: PublicKey;
  sellingResource: PublicKey;
  tradeHistory: PublicKey;
  tradeHistoryBump: number;
  market: PublicKey;
  marketTreasuryHolder: PublicKey;
  vaultOwner: PublicKey;
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
  resourceMintMetadata,
  resourceMintEditionMarker,
  resourceMintMasterEdition,
  sellingResource,
  tradeHistory,
  tradeHistoryBump,
  market,
  marketTreasuryHolder,
  vault,
  vaultOwner,
  vaultOwnerBump,
  newMint,
  newMintEdition,
  newMintMetadata,
}: BuyParams) => {
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
      // resource mint metadata PDA
      masterEditionMetadata: resourceMintMetadata,
      // token account for selling resource
      vault,
      // account which holds selling entities
      sellingResource,
      // owner of selling resource token account PDA
      owner: vaultOwner,
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
