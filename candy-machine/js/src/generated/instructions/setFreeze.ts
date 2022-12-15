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
 * @category SetFreeze
 * @category generated
 */
export type SetFreezeInstructionArgs = {
  freezeTime: beet.bignum;
};
/**
 * @category Instructions
 * @category SetFreeze
 * @category generated
 */
export const setFreezeStruct = new beet.BeetArgsStruct<
  SetFreezeInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['freezeTime', beet.i64],
  ],
  'SetFreezeInstructionArgs',
);
/**
 * Accounts required by the _setFreeze_ instruction
 *
 * @property [_writable_] candyMachine
 * @property [_writable_, **signer**] authority
 * @property [_writable_] freezePda
 * @category Instructions
 * @category SetFreeze
 * @category generated
 */
export type SetFreezeInstructionAccounts = {
  candyMachine: web3.PublicKey;
  authority: web3.PublicKey;
  freezePda: web3.PublicKey;
  systemProgram?: web3.PublicKey;
};

export const setFreezeInstructionDiscriminator = [202, 80, 109, 208, 130, 144, 26, 233];

/**
 * Creates a _SetFreeze_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category SetFreeze
 * @category generated
 */
export function createSetFreezeInstruction(
  accounts: SetFreezeInstructionAccounts,
  args: SetFreezeInstructionArgs,
  programId = new web3.PublicKey('cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ'),
) {
  const [data] = setFreezeStruct.serialize({
    instructionDiscriminator: setFreezeInstructionDiscriminator,
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
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.freezePda,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
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
