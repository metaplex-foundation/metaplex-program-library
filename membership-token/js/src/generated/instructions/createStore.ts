import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import { DESCRIPTION_DEFAULT_SIZE, NAME_DEFAULT_SIZE } from '../consts';

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
    ['name', beet.fixedSizeUtf8String(NAME_DEFAULT_SIZE)],
    ['description', beet.fixedSizeUtf8String(DESCRIPTION_DEFAULT_SIZE)],
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
    programId: new web3.PublicKey('5KtWMwMnvTzF9Uqg7idUR43hdMhEbgKUwXX5ef9Wajrq'),
    keys,
    data,
  });
  return ix;
}
