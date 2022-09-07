/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * @category Instructions
 * @category Mint
 * @category generated
 */
export type MintInstructionArgs = {
  creatorBump: number;
  mintArgs: Uint8Array;
};
/**
 * @category Instructions
 * @category Mint
 * @category generated
 */
export const mintStruct = new beet.FixableBeetArgsStruct<
  MintInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['creatorBump', beet.u8],
    ['mintArgs', beet.bytes],
  ],
  'MintInstructionArgs',
);
/**
 * Accounts required by the _mint_ instruction
 *
 * @property [] candyGuard
 * @property [] candyMachineProgram
 * @property [_writable_] candyMachine
 * @property [] updateAuthority
 * @property [_writable_] candyMachineCreator
 * @property [_writable_, **signer**] payer
 * @property [_writable_] metadata
 * @property [_writable_] mint
 * @property [**signer**] mintAuthority
 * @property [**signer**] mintUpdateAuthority
 * @property [_writable_] masterEdition
 * @property [] tokenMetadataProgram
 * @property [] recentSlothashes
 * @property [] instructionSysvarAccount
 * @category Instructions
 * @category Mint
 * @category generated
 */
export type MintInstructionAccounts = {
  candyGuard: web3.PublicKey;
  candyMachineProgram: web3.PublicKey;
  candyMachine: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  candyMachineCreator: web3.PublicKey;
  payer: web3.PublicKey;
  metadata: web3.PublicKey;
  mint: web3.PublicKey;
  mintAuthority: web3.PublicKey;
  mintUpdateAuthority: web3.PublicKey;
  masterEdition: web3.PublicKey;
  tokenMetadataProgram: web3.PublicKey;
  tokenProgram?: web3.PublicKey;
  systemProgram?: web3.PublicKey;
  rent?: web3.PublicKey;
  recentSlothashes: web3.PublicKey;
  instructionSysvarAccount: web3.PublicKey;
};

export const mintInstructionDiscriminator = [51, 57, 225, 47, 182, 146, 137, 166];

/**
 * Creates a _Mint_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category Mint
 * @category generated
 */
export function createMintInstruction(
  accounts: MintInstructionAccounts,
  args: MintInstructionArgs,
  programId = new web3.PublicKey('grd1hVewsa8dR1T1JfSFGzQUqgWmc1xXZ3uRRFJJ8XJ'),
) {
  const [data] = mintStruct.serialize({
    instructionDiscriminator: mintInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.candyGuard,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.candyMachineProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.candyMachine,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.updateAuthority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.candyMachineCreator,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.payer,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.mint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.mintAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.mintUpdateAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.masterEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenMetadataProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenProgram ?? splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.rent ?? web3.SYSVAR_RENT_PUBKEY,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.recentSlothashes,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.instructionSysvarAccount,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}
