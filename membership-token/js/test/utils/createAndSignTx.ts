import { Connection, Keypair, Signer, Transaction, TransactionInstruction } from '@solana/web3.js';

export async function createAndSignTransaction(
  instruction: TransactionInstruction,
  connection: Connection,
  payer: Keypair,
  signers: Signer[],
): Promise<Transaction> {
  const tx = new Transaction();
  tx.add(instruction);
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  tx.feePayer = payer.publicKey;
  tx.partialSign(...signers);

  return tx;
}
