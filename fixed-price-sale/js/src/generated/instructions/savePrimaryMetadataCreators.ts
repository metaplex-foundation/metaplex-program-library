import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

import { CreatorAccountData, creatorAccountDataStruct, PROGRAM_ID } from '../..';

export type SavePrimaryMetadataCreatorsInstructionArgs = {
  primaryMetadataCreatorsBump: number;
  creators: CreatorAccountData[];
};
const savePrimaryMetadataCreatorsStruct = new beet.FixableBeetArgsStruct<
  SavePrimaryMetadataCreatorsInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['primaryMetadataCreatorsBump', beet.u8],
    ['creators', beet.array(creatorAccountDataStruct)],
  ],
  'SavePrimaryMetadataCreatorsInstructionArgs',
);
export type SavePrimaryMetadataCreatorsInstructionAccounts = {
  admin: web3.PublicKey;
  metadata: web3.PublicKey;
  primaryMetadataCreators: web3.PublicKey;
};

const savePrimaryMetadataCreatorsInstructionDiscriminator = [66, 240, 213, 46, 185, 60, 192, 254];

/**
 * Creates a _SavePrimaryMetadataCreators_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createSavePrimaryMetadataCreatorsInstruction(
  accounts: SavePrimaryMetadataCreatorsInstructionAccounts,
  args: SavePrimaryMetadataCreatorsInstructionArgs,
) {
  const { admin, metadata, primaryMetadataCreators } = accounts;

  const [data] = savePrimaryMetadataCreatorsStruct.serialize({
    instructionDiscriminator: savePrimaryMetadataCreatorsInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: admin,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: primaryMetadataCreators,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey(PROGRAM_ID),
    keys,
    data,
  });
  return ix;
}
