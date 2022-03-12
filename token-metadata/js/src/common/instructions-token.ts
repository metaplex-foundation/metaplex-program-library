import {
  Account as TokenAccount,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';

// prettier-ignore
import {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore thes are exported, just not properly setup
  createAssociatedTokenAccountInstruction, TokenAccountNotFoundError, TokenInvalidAccountOwnerError, getAssociatedTokenAddress
} from '@solana/spl-token';

import { Commitment, Connection, PublicKey, TransactionInstruction } from '@solana/web3.js';
import { strict as assert } from 'assert';

/**
 * Retrieve the associated token account, or provides the instruction to create
 * it if it doesn't exist
 * Derived from spl-token/js/src/actions/getOrCreateAssociatedTokenAccount.ts but returning an instruction
 * instead of a transaction.
 *
 * @param connection               Connection to use
 * @param payer                    Payer of the transaction and initialization fees
 * @param mint                     Mint associated with the account to set or verify
 * @param owner                    Owner of the account to set or verify
 * @param allowOwnerOffCurve       Allow the owner account to be a PDA (Program Derived Address)
 * @param commitment               Desired level of commitment for querying the state
 * @param programId                SPL Token program account
 * @param associatedTokenProgramId SPL Associated Token program account
 *
 * @return value the address of the associated account and if needed the
 * {@link TransactionInstruction} to create it
 */
export async function getOrCreateAssociatedTokenAccountInstruction(
  connection: Connection,
  payer: PublicKey,
  mint: PublicKey,
  owner: PublicKey,
  allowOwnerOffCurve = false,
  commitment?: Commitment,
  programId = TOKEN_PROGRAM_ID,
  associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID,
): Promise<{
  instruction?: TransactionInstruction;
  ataAddress: PublicKey;
}> {
  const ataAddress = await getAssociatedTokenAddress(
    mint,
    owner,
    allowOwnerOffCurve,
    associatedTokenProgramId,
    programId,
  );

  // This is the optimal logic, considering TX fee, client-side computation, RPC roundtrips and guaranteed idempotent.
  // Sadly we can't do this atomically.
  let account: TokenAccount;
  try {
    account = await getAccount(connection, ataAddress, commitment, programId);
  } catch (err) {
    if (err instanceof TokenAccountNotFoundError || err instanceof TokenInvalidAccountOwnerError) {
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
    } else {
      throw err;
    }
  }

  assert(account.mint.equals(mint), 'TokenInvalidMintError');
  assert(account.owner.equals(owner), 'TokenInvalidOwnerError');

  return {
    ataAddress: ataAddress,
  };
}
