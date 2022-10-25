/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'

/**
 * @category Instructions
 * @category SetTreeDelegate
 * @category generated
 */
export const setTreeDelegateStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[] /* size: 8 */
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'SetTreeDelegateInstructionArgs'
)
/**
 * Accounts required by the _setTreeDelegate_ instruction
 *
 * @property [_writable_] treeAuthority
 * @property [**signer**] treeCreator
 * @property [] newTreeDelegate
 * @property [] merkleTree
 * @category Instructions
 * @category SetTreeDelegate
 * @category generated
 */
export type SetTreeDelegateInstructionAccounts = {
  treeAuthority: web3.PublicKey
  treeCreator: web3.PublicKey
  newTreeDelegate: web3.PublicKey
  merkleTree: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const setTreeDelegateInstructionDiscriminator = [
  253, 118, 66, 37, 190, 49, 154, 102,
]

/**
 * Creates a _SetTreeDelegate_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category SetTreeDelegate
 * @category generated
 */
export function createSetTreeDelegateInstruction(
  accounts: SetTreeDelegateInstructionAccounts,
  programId = new web3.PublicKey('BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY')
) {
  const [data] = setTreeDelegateStruct.serialize({
    instructionDiscriminator: setTreeDelegateInstructionDiscriminator,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.treeAuthority,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.treeCreator,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.newTreeDelegate,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.merkleTree,
      isWritable: false,
      isSigner: false,
    },
  ]

  if (accounts.anchorRemainingAccounts != null) {
    for (const acc of accounts.anchorRemainingAccounts) {
      keys.push(acc)
    }
  }

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  })
  return ix
}
