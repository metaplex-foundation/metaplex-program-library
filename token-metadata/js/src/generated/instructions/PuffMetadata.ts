/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * @category Instructions
 * @category PuffMetadata
 * @category generated
 */
export const PuffMetadataStruct = new beet.BeetArgsStruct<{ instructionDiscriminator: number }>(
  [['instructionDiscriminator', beet.u8]],
  'PuffMetadataInstructionArgs',
);
/**
 * Accounts required by the _PuffMetadata_ instruction
 *
 * @property [_writable_] metadata Metadata account
 * @property [] sysvarInstructions Instructions sysvar account
 * @category Instructions
 * @category PuffMetadata
 * @category generated
 */
export type PuffMetadataInstructionAccounts = {
  metadata: web3.PublicKey;
  systemProgram?: web3.PublicKey;
  sysvarInstructions: web3.PublicKey;
};

export const puffMetadataInstructionDiscriminator = 14;

/**
 * Creates a _PuffMetadata_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category PuffMetadata
 * @category generated
 */
export function createPuffMetadataInstruction(
  accounts: PuffMetadataInstructionAccounts,
  programId = new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
) {
  const [data] = PuffMetadataStruct.serialize({
    instructionDiscriminator: puffMetadataInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.metadata,
      isWritable: true,
      isSigner: false,
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
  ];

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}
