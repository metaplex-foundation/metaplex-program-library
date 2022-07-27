import { Amman } from '@metaplex-foundation/amman-client';
import { Wallet } from '@project-serum/anchor/dist/cjs/provider';
import { Connection, PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import * as splToken from '@solana/spl-token';
import { withFindOrInitAssociatedTokenAccount } from '@cardinal/token-manager';

export const amman = Amman.instance();

/**
 * Pay and create mint and token account
 * @param connection
 * @param creator
 * @returns
 */
export const createMintTransaction = async (
  transaction: Transaction,
  connection: Connection,
  wallet: Wallet,
  recipient: PublicKey,
  mintId: PublicKey,
  amount = 1,
  freezeAuthority: PublicKey = recipient,
): Promise<[PublicKey, Transaction]> => {
  const mintBalanceNeeded = await splToken.Token.getMinBalanceRentForExemptMint(connection);
  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: mintId,
      lamports: mintBalanceNeeded,
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      space: splToken.MintLayout.span,
      programId: splToken.TOKEN_PROGRAM_ID,
    }),
  );
  transaction.add(
    splToken.Token.createInitMintInstruction(
      splToken.TOKEN_PROGRAM_ID,
      mintId,
      0,
      wallet.publicKey,
      freezeAuthority,
    ),
  );
  const walletAta = await withFindOrInitAssociatedTokenAccount(
    transaction,
    connection,
    mintId,
    wallet.publicKey,
    wallet.publicKey,
  );
  if (amount > 0) {
    transaction.add(
      splToken.Token.createMintToInstruction(
        splToken.TOKEN_PROGRAM_ID,
        mintId,
        walletAta,
        wallet.publicKey,
        [],
        amount,
      ),
    );
  }
  return [walletAta, transaction];
};
