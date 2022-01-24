import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

import { PROGRAM_ID, DESCRIPTION_MAX_LEN, NAME_MAX_LEN } from '../consts';
import { checkByteSizes } from '../utils';

export type CreateStoreInstructionArgs = {
  name: string;
  description: string;
};
const createStoreStruct = new beet.BeetArgsStruct<
  CreateStoreInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.fixedSizeArray(beet.u8, 8)],
    ['name', beet.fixedSizeUtf8String(NAME_MAX_LEN)],
    ['description', beet.fixedSizeUtf8String(DESCRIPTION_MAX_LEN)],
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

  const name = checkByteSizes(args['name'], NAME_MAX_LEN);
  const description = checkByteSizes(args['description'], DESCRIPTION_MAX_LEN);

  Object.assign(args, { name, description });

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
