import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type UpdateMetadataAccountInstructionArgs = {
  updateMetadataAccountArgs: definedTypes.UpdateMetadataAccountArgs;
};
const UpdateMetadataAccountStruct = new beet.FixableBeetArgsStruct<
  UpdateMetadataAccountInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['updateMetadataAccountArgs', definedTypes.updateMetadataAccountArgsStruct],
  ],
  'UpdateMetadataAccountInstructionArgs',
);
export type UpdateMetadataAccountInstructionAccounts = {
  metadata: web3.PublicKey;
  updateAuthority: web3.PublicKey;
};

const updateMetadataAccountInstructionDiscriminator = [141, 14, 23, 104, 247, 192, 53, 173];

/**
 * Creates a _UpdateMetadataAccount_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createUpdateMetadataAccountInstruction(
  accounts: UpdateMetadataAccountInstructionAccounts,
  args: UpdateMetadataAccountInstructionArgs,
) {
  const { metadata, updateAuthority } = accounts;

  const [data] = UpdateMetadataAccountStruct.serialize({
    instructionDiscriminator: updateMetadataAccountInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: updateAuthority,
      isWritable: false,
      isSigner: true,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
    keys,
    data,
  });
  return ix;
}
