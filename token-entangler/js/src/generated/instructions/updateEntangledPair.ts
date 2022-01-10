import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type UpdateEntangledPairInstructionArgs = {
  price: beet.bignum;
  paysEveryTime: boolean;
};
const updateEntangledPairStruct = new beet.BeetArgsStruct<
  UpdateEntangledPairInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.fixedSizeArray(beet.u8, 8)],
    ['price', beet.u64],
    ['paysEveryTime', beet.bool],
  ],
  'UpdateEntangledPairInstructionArgs',
);
export type UpdateEntangledPairInstructionAccounts = {
  authority: web3.PublicKey;
  newAuthority: web3.PublicKey;
  entangledPair: web3.PublicKey;
};

const updateEntangledPairInstructionDiscriminator = [41, 97, 247, 218, 98, 162, 75, 244];

/**
 * Creates a _UpdateEntangledPair_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createUpdateEntangledPairInstruction(
  accounts: UpdateEntangledPairInstructionAccounts,
  args: UpdateEntangledPairInstructionArgs,
) {
  const { authority, newAuthority, entangledPair } = accounts;

  const [data] = updateEntangledPairStruct.serialize({
    instructionDiscriminator: updateEntangledPairInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: authority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: newAuthority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: entangledPair,
      isWritable: true,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('qntmGodpGkrM42mN68VCZHXnKqDCT8rdY23wFcXCLPd'),
    keys,
    data,
  });
  return ix;
}
