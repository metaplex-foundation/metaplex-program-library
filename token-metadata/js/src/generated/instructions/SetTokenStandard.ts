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
 * @category SetTokenStandard
 * @category generated
 */
export const SetTokenStandardStruct = new beet.BeetArgsStruct<{ instructionDiscriminator: number }>(
  [['instructionDiscriminator', beet.u8]],
  'SetTokenStandardInstructionArgs',
);
/**
 * Accounts required by the _SetTokenStandard_ instruction
 *
 * @property [_writable_] metadata Metadata account
 * @property [_writable_, **signer**] updateAuthority Metadata update authority
 * @property [] mint Mint account
 * @property [] edition (optional) Edition account
 * @category Instructions
 * @category SetTokenStandard
 * @category generated
 */
export type SetTokenStandardInstructionAccounts = {
  metadata: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  mint: web3.PublicKey;
  edition?: web3.PublicKey;
};

export const setTokenStandardInstructionDiscriminator = 36;

/**
 * Creates a _SetTokenStandard_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category SetTokenStandard
 * @category generated
 */
export function createSetTokenStandardInstruction(
  accounts: SetTokenStandardInstructionAccounts,
  programId = new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
) {
  const [data] = SetTokenStandardStruct.serialize({
    instructionDiscriminator: setTokenStandardInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.updateAuthority,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.mint,
      isWritable: false,
      isSigner: false,
    },
  ];

  if (accounts.edition != null) {
    keys.push({
      pubkey: accounts.edition,
      isWritable: false,
      isSigner: false,
    });
  }

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}
