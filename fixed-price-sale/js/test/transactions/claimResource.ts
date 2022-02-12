import { MetadataProgram } from '@metaplex-foundation/mpl-token-metadata';
import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import { createClaimResourceInstruction } from '../../src/instructions';
import { createAndSignTransaction } from '../utils';

interface ClaimResourceParams {
  payer: Keypair;
  connection: Connection;
  market: PublicKey;
  treasuryHolder: PublicKey;
  sellingResource: PublicKey;
  vault: PublicKey;
  metadata: PublicKey;
  secondaryMetadataCreators: PublicKey;
  destination: PublicKey;
  vaultOwnerBump: number;
  owner: PublicKey;
}

export const createClaimResourceTransaction = async ({
  payer,
  connection,
  market,
  treasuryHolder,
  sellingResource,
  vault,
  metadata,
  secondaryMetadataCreators,
  destination,
  vaultOwnerBump,
  owner,
}: ClaimResourceParams): Promise<Transaction> => {
  const instruction = await createClaimResourceInstruction(
    {
      market,
      treasuryHolder,
      sellingResource,
      sellingResourceOwner: payer.publicKey,
      vault,
      metadata,
      owner,
      secondaryMetadataCreators,
      destination,
      tokenMetadataProgram: MetadataProgram.PUBKEY,
    },
    {
      vaultOwnerBump,
    },
  );

  const claimResourceTx: Transaction = await createAndSignTransaction(
    connection,
    payer,
    [instruction],
    [payer],
  );

  return claimResourceTx;
};
