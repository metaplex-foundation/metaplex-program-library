import { MintLayout, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';

/**
 * Allocates an account for the provided {@link mint} address.
 * Used by setups that need to initialize a mint account.
 *
 * @param args
 * @param args.payer funds transaction
 * @param args.mint the mint address
 * @param args.decimals number of decimals in token account amounts
 * @param args.owner mint owner, defaults to {@link args.payer}
 * @param args.freezeAuthority mint owner, defaults to {@link args.payer}
 *
 * @category common
 * @private
 */
export async function createMintInstructions(
  connection: Connection,
  args: {
    payer: PublicKey;
    mint: PublicKey;
    decimals?: number;
    owner?: PublicKey;
    freezeAuthority?: PublicKey;
  },
) {
  const { payer, mint, decimals = 0, owner = args.payer, freezeAuthority = args.payer } = args;
  const mintRent = await connection.getMinimumBalanceForRentExemption(MintLayout.span, 'confirmed');
  return [
    SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: mint,
      lamports: mintRent,
      space: MintLayout.span,
      programId: TOKEN_PROGRAM_ID,
    }),
    Token.createInitMintInstruction(TOKEN_PROGRAM_ID, mint, decimals, owner, freezeAuthority),
  ];
}
