/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * @category Instructions
 * @category UpdateAuthority
 * @category generated
 */
export type UpdateAuthorityInstructionArgs = {
  newAuthority: beet.COption<web3.PublicKey>;
};
/**
 * @category Instructions
 * @category UpdateAuthority
 * @category generated
 */
export const updateAuthorityStruct = new beet.FixableBeetArgsStruct<
  UpdateAuthorityInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['newAuthority', beet.coption(beetSolana.publicKey)],
  ],
  'UpdateAuthorityInstructionArgs',
);
/**
 * Accounts required by the _updateAuthority_ instruction
 *
 * @property [_writable_] candyMachine
 * @property [**signer**] authority
 * @property [] wallet
 * @category Instructions
 * @category UpdateAuthority
 * @category generated
 */
export type UpdateAuthorityInstructionAccounts = {
  candyMachine: web3.PublicKey;
  authority: web3.PublicKey;
  wallet: web3.PublicKey;
  anchorRemainingAccounts?: web3.AccountMeta[];
};

export const updateAuthorityInstructionDiscriminator = [32, 46, 64, 28, 149, 75, 243, 88];

/**
 * Creates a _UpdateAuthority_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category UpdateAuthority
 * @category generated
 */
export function createUpdateAuthorityInstruction(
  accounts: UpdateAuthorityInstructionAccounts,
  args: UpdateAuthorityInstructionArgs,
  programId = new web3.PublicKey('cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ'),
) {
  const [data] = updateAuthorityStruct.serialize({
    instructionDiscriminator: updateAuthorityInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.candyMachine,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.authority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.wallet,
      isWritable: false,
      isSigner: false,
    },
  ];

  if (accounts.anchorRemainingAccounts != null) {
    for (const acc of accounts.anchorRemainingAccounts) {
      keys.push(acc);
    }
  }

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}