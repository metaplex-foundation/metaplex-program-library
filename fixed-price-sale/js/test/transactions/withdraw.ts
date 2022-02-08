import { ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import { createAndSignTransaction } from '../utils';
import { createWithdrawInstruction } from '../../src/instructions';

interface WithdrawParams {
  payer: Keypair;
  connection: Connection;
  market: PublicKey;
  payoutTicket: PublicKey;
  destination: PublicKey;
  treasuryMint: PublicKey;
  treasuryHolder: PublicKey;
  metadata: PublicKey;
  sellingResource: PublicKey;
  payoutTicketBump: number;
  treasuryOwnerBump: number;
  treasuryOwner: PublicKey;
}

export const createWithdrawTransaction = async ({
  payer,
  connection,
  market,
  payoutTicket,
  destination,
  treasuryMint,
  treasuryHolder,
  metadata,
  sellingResource,
  treasuryOwnerBump,
  payoutTicketBump,
  treasuryOwner,
}: WithdrawParams): Promise<Transaction> => {
  const instruction = await createWithdrawInstruction(
    {
      market,
      sellingResource,
      metadata,
      treasuryHolder,
      treasuryMint,
      owner: treasuryOwner,
      destination,
      funder: payer.publicKey,
      payer: payer.publicKey,
      payoutTicket: payoutTicket,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    },
    {
      treasuryOwnerBump,
      payoutTicketBump,
    },
  );

  const withdrawTx: Transaction = await createAndSignTransaction(
    connection,
    payer,
    [instruction],
    [payer],
  );

  return withdrawTx;
};
