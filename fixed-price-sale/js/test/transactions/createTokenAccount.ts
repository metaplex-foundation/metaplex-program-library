import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import {
  AccountLayout,
  createInitializeAccountInstruction,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';

export const createTokenAccount = async ({
  payer,
  mint,
  connection,
  owner,
}: {
  payer: PublicKey;
  mint: PublicKey;
  connection: Connection;
  owner?: PublicKey;
}) => {
  const tokenAccount = Keypair.generate();

  const createTokenTx = new Transaction();

  const accountRentExempt = await connection.getMinimumBalanceForRentExemption(AccountLayout.span);

  createTokenTx.add(
    SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: tokenAccount.publicKey,
      lamports: accountRentExempt,
      space: AccountLayout.span,
      programId: new PublicKey(TOKEN_PROGRAM_ID),
    }),
  );

  createTokenTx.add(
    createInitializeAccountInstruction(tokenAccount.publicKey, mint, owner ?? payer),
  );

  createTokenTx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  createTokenTx.feePayer = payer;
  createTokenTx.partialSign(tokenAccount);

  return {
    tokenAccount,
    createTokenTx,
  };
};
