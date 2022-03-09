import { BeetArgsStruct, bignum, u64, u8 } from '@metaplex-foundation/beet';
import { MintLayout, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  AccountMeta,
  Connection,
  PublicKey,
  Signer,
  SystemProgram,
  TransactionInstruction,
} from '@solana/web3.js';
import { getOrCreateAssociatedTokenAccountIntruction as getOrCreateAssociatedTokenAccountInfoIntructions } from './instructions-token';

// NOTE: lots of these where pulled from the spl-token/ts folder and should
// be used from there once we update the version of that lib

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
    Token.createInitMintInstruction(TOKEN_PROGRAM_ID, mint, decimals, owner, freezeAuthority),
  ];
  if (associateWithOwner) {
    const { ataAddress: address, instruction: createAssociatedTokenAccountIx } =
      await getOrCreateAssociatedTokenAccountInfoIntructions(connection, args.payer, mint, owner);
    if (createAssociatedTokenAccountIx != null) {
      instructions.push(createAssociatedTokenAccountIx);
    }
    const mintToIx = createMintToInstruction(mint, address, payer, 1);
    instructions.push(mintToIx);
  }
  return instructions;
}

const MintTo = 7;
export type MintToInstructionData = {
  instruction: number;
  amount: bignum;
};
export const mintToInstructionData = new BeetArgsStruct<MintToInstructionData>([
  ['instruction', u8],
  ['amount', u64],
]);
/**
 * Construct a MintTo instruction
 *
 * @param mint         Public key of the mint
 * @param destination  Address of the token account to mint to
 * @param authority    The mint authority
 * @param amount       Amount to mint
 * @param multiSigners Signing accounts if `authority` is a multisig
 * @param programId    SPL Token program account
 *
 * @return Instruction to add to a transaction
 */
export function createMintToInstruction(
  mint: PublicKey,
  destination: PublicKey,
  authority: PublicKey,
  amount: bignum,
  multiSigners: Signer[] = [],
  programId = TOKEN_PROGRAM_ID,
): TransactionInstruction {
  const keys = addSigners(
    [
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: destination, isSigner: false, isWritable: true },
    ],
    authority,
    multiSigners,
  );

  const [data] = mintToInstructionData.serialize(
    {
      instruction: MintTo,
      amount,
    },
    mintToInstructionData.byteSize,
  );

  return new TransactionInstruction({ keys, programId, data });
}

// -----------------
// Helpers
// -----------------
function addSigners(
  keys: AccountMeta[],
  ownerOrAuthority: PublicKey,
  multiSigners: Signer[],
): AccountMeta[] {
  if (multiSigners.length) {
    keys.push({ pubkey: ownerOrAuthority, isSigner: false, isWritable: false });
    for (const signer of multiSigners) {
      keys.push({ pubkey: signer.publicKey, isSigner: true, isWritable: false });
    }
  } else {
    keys.push({ pubkey: ownerOrAuthority, isSigner: true, isWritable: false });
  }
  return keys;
}
