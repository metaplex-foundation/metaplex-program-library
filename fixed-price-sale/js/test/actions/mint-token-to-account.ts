import { Connection, PublicKey, Transaction } from '@solana/web3.js';
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Actions, defaultSendOptions, TransactionHandler } from '@metaplex-foundation/amman';

import { createTokenAccount } from '../transactions';

interface MintTokenToAccountParams {
  connection: Connection;
  payer: PublicKey;
  transactionHandler: TransactionHandler;
}

export const mintTokenToAccount = async ({
  connection,
  payer,
  transactionHandler,
}: MintTokenToAccountParams) => {
  const tx = new Transaction();

  const { mint, createMintTx } = await new Actions(connection).createMintAccount(payer);

  tx.add(createMintTx);

  const { tokenAccount: associatedTokenAccount, createTokenTx } = await createTokenAccount({
    payer,
    mint: mint.publicKey,
    connection,
  });

  tx.add(createTokenTx);

  tx.add(
    Token.createMintToInstruction(
      new PublicKey(TOKEN_PROGRAM_ID),
      mint.publicKey,
      associatedTokenAccount.publicKey,
      payer,
      [],
      1,
    ),
  );

  await transactionHandler.sendAndConfirmTransaction(
    tx,
    [mint, associatedTokenAccount],
    defaultSendOptions,
  );

  return { mint, mintAta: associatedTokenAccount };
};
