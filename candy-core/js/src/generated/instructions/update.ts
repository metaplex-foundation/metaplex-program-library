/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'
import {
  CandyMachineData,
  candyMachineDataBeet,
} from '../types/CandyMachineData'

/**
 * @category Instructions
 * @category Update
 * @category generated
 */
export type UpdateInstructionArgs = {
  data: CandyMachineData
}
/**
 * @category Instructions
 * @category Update
 * @category generated
 */
export const updateStruct = new beet.FixableBeetArgsStruct<
  UpdateInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['data', candyMachineDataBeet],
  ],
  'UpdateInstructionArgs'
)
/**
 * Accounts required by the _update_ instruction
 *
 * @property [_writable_] candyMachine
 * @property [**signer**] authority
 * @property [] wallet
 * @category Instructions
 * @category Update
 * @category generated
 */
export type UpdateInstructionAccounts = {
  candyMachine: web3.PublicKey
  authority: web3.PublicKey
  wallet: web3.PublicKey
}

export const updateInstructionDiscriminator = [
  219, 200, 88, 176, 158, 63, 253, 127,
]

/**
 * Creates a _Update_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category Update
 * @category generated
 */
export function createUpdateInstruction(
  accounts: UpdateInstructionAccounts,
  args: UpdateInstructionArgs,
  programId = new web3.PublicKey('cndy3CZK71ZHMp9ddpq5NVvQDx33o6cCYDf4JBAWCk7')
) {
  const [data] = updateStruct.serialize({
    instructionDiscriminator: updateInstructionDiscriminator,
    ...args,
  })
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
  ]

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  })
  return ix
}