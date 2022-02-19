import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type UpdateMetadataAccountV2InstructionArgs = {
  updateMetadataAccountArgsV2: definedTypes.UpdateMetadataAccountArgsV2;
};
const UpdateMetadataAccountV2Struct = new beet.FixableBeetArgsStruct<
  UpdateMetadataAccountV2InstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['updateMetadataAccountArgsV2', definedTypes.updateMetadataAccountArgsV2Struct],
  ],
  'UpdateMetadataAccountV2InstructionArgs',
);
export type UpdateMetadataAccountV2InstructionAccounts = {
  metadata: web3.PublicKey;
  updateAuthority: web3.PublicKey;
};

const updateMetadataAccountV2InstructionDiscriminator = [202, 132, 152, 229, 216, 217, 137, 212];

/**
 * Creates a _UpdateMetadataAccountV2_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createUpdateMetadataAccountV2Instruction(
  accounts: UpdateMetadataAccountV2InstructionAccounts,
  args: UpdateMetadataAccountV2InstructionArgs,
) {
  const { metadata, updateAuthority } = accounts;

  const [data] = UpdateMetadataAccountV2Struct.serialize({
    instructionDiscriminator: updateMetadataAccountV2InstructionDiscriminator,
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
