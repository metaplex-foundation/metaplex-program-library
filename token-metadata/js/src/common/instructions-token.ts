import {
  AccountInfo as TokenAccountInfo,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import {
  Connection,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import { strict as assert } from 'assert';

// NOTE: lots of these where pulled from the spl-token/ts folder and should
// be used from there once we update the version of that lib

/**
 * Construct an AssociatedTokenAccount instruction
 *
 * @param payer                    Payer of the initialization fees
 * @param associatedToken          New associated token account
 * @param owner                    Owner of the new account
 * @param mint                     Token mint account
 * @param programId                SPL Token program account
 * @param associatedTokenProgramId SPL Associated Token program account
 *
 * @return Instruction to add to a transaction
 */
export function createAssociatedTokenAccountInstruction(
  payer: PublicKey,
  associatedToken: PublicKey,
  owner: PublicKey,
  mint: PublicKey,
  programId = TOKEN_PROGRAM_ID,
  associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID,
): TransactionInstruction {
  const keys = [
    { pubkey: payer, isSigner: true, isWritable: true },
    { pubkey: associatedToken, isSigner: false, isWritable: true },
    { pubkey: owner, isSigner: false, isWritable: false },
    { pubkey: mint, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    { pubkey: programId, isSigner: false, isWritable: false },
    { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
  ];

  return new TransactionInstruction({
    keys,
    programId: associatedTokenProgramId,
    data: Buffer.alloc(0),
  });
}

/**
 * Retrieve the associated token account, or create it if it doesn't exist
 *
 * @param connection               Connection to use
 * @param payer                    Payer of the transaction and initialization fees
 * @param mint                     Mint associated with the account to set or verify
 * @param owner                    Owner of the account to set or verify
 * @param allowOwnerOffCurve       Allow the owner account to be a PDA (Program Derived Address)
 * @param commitment               Desired level of commitment for querying the state
 * @param confirmOptions           Options for confirming the transaction
 * @param programId                SPL Token program account
 * @param associatedTokenProgramId SPL Associated Token program account
 *
 * @return value the address of the associated account and if needed the
 * {@link TransactionInstruction} to create it
 */
export async function getOrCreateAssociatedTokenAccountIntruction(
  connection: Connection,
  payer: PublicKey,
  mint: PublicKey,
  owner: PublicKey,
  allowOwnerOffCurve = false,
  programId = TOKEN_PROGRAM_ID,
  associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID,
): Promise<{
  instruction?: TransactionInstruction;
  ataAddress: PublicKey;
}> {
  const ataAddress = await Token.getAssociatedTokenAddress(
    associatedTokenProgramId,
    programId,
    mint,
    owner,
    allowOwnerOffCurve,
  );

  // This is the optimal logic, considering TX fee, client-side computation, RPC roundtrips and guaranteed idempotent.
  // Sadly we can't do this atomically.
  let account: TokenAccountInfo;
  try {
    // NOTE: this now imroved Token API requires us to provide a payer even though it is not used
    // by `getAccountInfo` which is all we want to use
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore missing signer
    account = await new Token(connection, mint, programId).getAccountInfo(ataAddress);
  } catch (_) {
    // TODO(thlorenz): This is very simplified as we don't consider the below, but always assume
    // that the account still needs to be created if we don't find it.
    //   TokenAccountNotFoundError can be possible if the associated address has already received some lamports,
    //   becoming a system account. Assuming program derived addressing is safe, this is the only case for the
    //   TokenInvalidAccountOwnerError in this code path.
    return {
      ataAddress: ataAddress,
      instruction: createAssociatedTokenAccountInstruction(
        payer,
        ataAddress,
        owner,
        mint,
        programId,
        associatedTokenProgramId,
      ),
    };
  }

  assert(account.mint.equals(mint), 'TokenInvalidMintError');
  assert(account.owner.equals(owner), 'TokenInvalidOwnerError');

  return {
    ataAddress: ataAddress,
  };
}
