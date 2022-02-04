import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

import { PROGRAM_ID } from '../consts';

export type CreateStoreInstructionArgs = {
  name: string;
  description: string;
};
const createStoreStruct = new beet.FixableBeetArgsStruct<
  CreateStoreInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['name', beet.utf8String],
    ['description', beet.utf8String],
  ],
  'CreateStoreInstructionArgs',
);
export type CreateStoreInstructionAccounts = {
  admin: web3.PublicKey;
  store: web3.PublicKey;
};

const createStoreInstructionDiscriminator = [132, 152, 9, 27, 112, 19, 95, 83];

/**
 * Creates a _CreateStore_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createCreateStoreInstruction(
  accounts: CreateStoreInstructionAccounts,
  args: CreateStoreInstructionArgs,
) {
  const { admin, store } = accounts;

  const [data] = createStoreStruct.serialize({
    instructionDiscriminator: createStoreInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: admin,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: store,
      isWritable: true,
      isSigner: true,
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
