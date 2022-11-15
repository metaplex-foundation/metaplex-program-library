import { Connection, PublicKey, Transaction } from '@solana/web3.js';
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore createMintToInstruction export actually exist but isn't setup correctly
import { createMintToInstruction } from '@solana/spl-token';
import { PayerTransactionHandler } from '@metaplex-foundation/amman-client';
import { strict as assert } from 'assert';

import { CreateMint } from './createMintAccount';
import { createTokenAccount } from '../transactions';

interface MintTokenToAccountParams {
  connection: Connection;
  payer: PublicKey;
  transactionHandler: PayerTransactionHandler;
}

export const mintTokenToAccount = async ({
  connection,
  payer,
  transactionHandler,
}: MintTokenToAccountParams) => {
  const tx = new Transaction();

  const { mint, createMintTx } = await CreateMint.createMintAccount(connection, payer);

  tx.add(createMintTx);

  const { tokenAccount: associatedTokenAccount, createTokenTx } = await createTokenAccount({
    payer,
    mint: mint.publicKey,
    connection,
  });

  tx.add(createTokenTx);

  tx.add(createMintToInstruction(mint.publicKey, associatedTokenAccount.publicKey, payer, 1));

  await transactionHandler
    .sendAndConfirmTransaction(tx, [mint, associatedTokenAccount])
    .assertSuccess(assert);

  return { mint, mintAta: associatedTokenAccount };
};
