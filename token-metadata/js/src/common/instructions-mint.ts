import {
  createInitializeMintInstruction,
  createMintToInstruction,
  MintLayout,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';
import { getOrCreateAssociatedTokenAccountInstruction } from './instructions-token';

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
    associateWithOwner?: boolean;
  },
) {
  const {
    payer,
    mint,
    decimals = 0,
    owner = args.payer,
    freezeAuthority = args.payer,
    associateWithOwner = false,
  } = args;
  const mintRent = await connection.getMinimumBalanceForRentExemption(MintLayout.span, 'confirmed');
  const instructions = [
    SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: mint,
      lamports: mintRent,
      space: MintLayout.span,
      programId: TOKEN_PROGRAM_ID,
    }),
    createInitializeMintInstruction(mint, decimals, owner, freezeAuthority),
  ];
  if (associateWithOwner) {
    const { ataAddress: address, instruction: createAssociatedTokenAccountIx } =
      await getOrCreateAssociatedTokenAccountInstruction(connection, args.payer, mint, owner);
    if (createAssociatedTokenAccountIx != null) {
      instructions.push(createAssociatedTokenAccountIx);
    }
    const mintToIx = createMintToInstruction(mint, address, payer, 1);
    instructions.push(mintToIx);
  }
  return instructions;
}
