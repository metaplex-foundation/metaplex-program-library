import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

export type AddConfigLinesInstructionArgs = {
  index: number;
  configLines: definedTypes.ConfigLine[];
};
const addConfigLinesStruct = new beet.FixableBeetArgsStruct<
  AddConfigLinesInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['index', beet.u32],
    ['configLines', beet.array(definedTypes.configLineStruct)],
  ],
  'AddConfigLinesInstructionArgs',
);
/**
 * Accounts required by the _addConfigLines_ instruction
 */
export type AddConfigLinesInstructionAccounts = {
  candyMachine: web3.PublicKey;
  authority: web3.PublicKey;
};

const addConfigLinesInstructionDiscriminator = [223, 50, 224, 227, 151, 8, 115, 106];

/**
 * Creates a _AddConfigLines_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createAddConfigLinesInstruction(
  accounts: AddConfigLinesInstructionAccounts,
  args: AddConfigLinesInstructionArgs,
) {
  const { candyMachine, authority } = accounts;

  const [data] = addConfigLinesStruct.serialize({
    instructionDiscriminator: addConfigLinesInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: candyMachine,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: authority,
      isWritable: false,
      isSigner: true,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ'),
    keys,
    data,
  });
  return ix;
}
