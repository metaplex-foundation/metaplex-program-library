import { Connection, PublicKey, Transaction, SYSVAR_CLOCK_PUBKEY } from '@solana/web3.js';
import { MetadataProgram } from '@metaplex-foundation/mpl-token-metadata';

import { createBuyInstruction } from '../../src/instructions';
import { createAndSignTransaction } from '../utils';

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
  vaultOwner: PublicKey;
  vaultOwnerBump: number;
  resourceMint: PublicKey;
  resourceMintEdition: PublicKey;
  resourceMintEditionMarker: PublicKey;
  resourceMintMetadata: PublicKey;
  resourceMintMasterEdition: PublicKey;
  resourceMintMasterMetadata: PublicKey;
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
  vaultOwner,
  vaultOwnerBump,
  resourceMint,
  resourceMintEdition,
  resourceMintEditionMarker,
  resourceMintMetadata,
  resourceMintMasterEdition,
  resourceMintMasterMetadata,
}: BuyMembershipTokenParams) => {
  const instruction = await createBuyInstruction(
    {
      // market account
      market,
      // account which holds selling entities
      sellingResource,
      // buyer token account
      userTokenAccount: buyerTokenAccount,
      // buyer wallet
      userWallet: buyer,
      // PDA which creates on market for each buyer
      tradeHistory,
      // market treasury holder (buyer will send tokens to this account)
      treasuryHolder: marketTreasuryHolder,
      // newly generated mint metadata PDA
      newMetadata: resourceMintMetadata,
      // newly generated mint edition PDA
      newEdition: resourceMintEdition,
      // master edition for newly generated mint (genesis mint)
      masterEdition: resourceMintMasterEdition,
      // newly generated mint address
      newMint: resourceMint,
      // PDA which will be taken by newMint + newEdition
      editionMarker: resourceMintEditionMarker,
      // token account for selling resource
      vault,
      // owner of selling resource token account PDA
      owner: vaultOwner,
      // master edition metadata PDA (genesis mint metadata)
      masterEditionMetadata: resourceMintMasterMetadata,
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
