/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { DelegateArgs, delegateArgsBeet } from '../types/DelegateArgs';

/**
 * @category Instructions
 * @category Delegate
 * @category generated
 */
export type DelegateInstructionArgs = {
  delegateArgs: DelegateArgs;
};
/**
 * @category Instructions
 * @category Delegate
 * @category generated
 */
export const DelegateStruct = new beet.FixableBeetArgsStruct<
  DelegateInstructionArgs & {
    instructionDiscriminator: number;
  }
>(
  [
    ['instructionDiscriminator', beet.u8],
    ['delegateArgs', delegateArgsBeet],
  ],
  'DelegateInstructionArgs',
);
/**
 * Accounts required by the _Delegate_ instruction
 *
 * @property [_writable_] delegateRecord Delegate Record account
 * @property [] delegate Owner of the delegated account
 * @property [_writable_] metadata Metadata account
 * @property [] masterEdition (optional) Master Edition account
 * @property [] mint Mint of metadata
 * @property [_writable_] token (optional) Token account of mint
 * @property [**signer**] approver Approver (update authority or token owner) for the delegation
 * @property [_writable_, **signer**] payer Payer
 * @property [] sysvarInstructions Instructions sysvar account
 * @property [] splTokenProgram (optional) SPL Token Program
 * @property [] authorizationRulesProgram (optional) Token Authorization Rules Program
 * @property [] authorizationRules (optional) Token Authorization Rules account
 * @category Instructions
 * @category Delegate
 * @category generated
 */
export type DelegateInstructionAccounts = {
  delegateRecord: web3.PublicKey;
  delegate: web3.PublicKey;
  metadata: web3.PublicKey;
  masterEdition?: web3.PublicKey;
  mint: web3.PublicKey;
  token?: web3.PublicKey;
  approver: web3.PublicKey;
  payer: web3.PublicKey;
  systemProgram?: web3.PublicKey;
  sysvarInstructions: web3.PublicKey;
  splTokenProgram?: web3.PublicKey;
  authorizationRulesProgram?: web3.PublicKey;
  authorizationRules?: web3.PublicKey;
};

export const delegateInstructionDiscriminator = 48;

/**
 * Creates a _Delegate_ instruction.
 *
 * Optional accounts that are not provided default to the program ID since
 * this was indicated in the IDL from which this instruction was generated.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category Delegate
 * @category generated
 */
export function createDelegateInstruction(
  accounts: DelegateInstructionAccounts,
  args: DelegateInstructionArgs,
  programId = new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
) {
  const [data] = DelegateStruct.serialize({
    instructionDiscriminator: delegateInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.delegateRecord,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.delegate,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.masterEdition ?? programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.mint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.token ?? programId,
      isWritable: accounts.token != null,
      isSigner: false,
    },
    {
      pubkey: accounts.approver,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.payer,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.sysvarInstructions,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.splTokenProgram ?? programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.authorizationRulesProgram ?? programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.authorizationRules ?? programId,
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
