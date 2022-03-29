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
const PuffMetadataStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number;
}>([['instructionDiscriminator', beet.u8]], 'PuffMetadataInstructionArgs');
/**
 * Accounts required by the _PuffMetadata_ instruction
 *
 * @property [_writable_] metadata Metadata account
 * @category Instructions
 * @category PuffMetadata
 * @category generated
 */
export type PuffMetadataInstructionAccounts = {
  metadata: web3.PublicKey;
};

const puffMetadataInstructionDiscriminator = 14;

/**
 * Creates a _PuffMetadata_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 *
 * @category Instructions
 * @category PuffMetadata
 * @category generated
 */
export function createPuffMetadataInstruction(accounts: PuffMetadataInstructionAccounts) {
  const { metadata } = accounts;

  const [data] = PuffMetadataStruct.serialize({
    instructionDiscriminator: puffMetadataInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
    keys,
    data,
  });
  return ix;
}
