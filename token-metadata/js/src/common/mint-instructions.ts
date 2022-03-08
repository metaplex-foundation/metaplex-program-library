import { BeetArgsStruct, bignum, u64, u8 } from '@metaplex-foundation/beet';
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  AccountMeta,
  PublicKey,
  Signer,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';

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
