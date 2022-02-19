import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type CreateMetadataAccountV2InstructionArgs = {
  createMetadataAccountArgsV2: definedTypes.CreateMetadataAccountArgsV2;
};
const CreateMetadataAccountV2Struct = new beet.BeetArgsStruct<
  CreateMetadataAccountV2InstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['createMetadataAccountArgsV2', definedTypes.createMetadataAccountArgsV2Struct],
  ],
  'CreateMetadataAccountV2InstructionArgs',
);
export type CreateMetadataAccountV2InstructionAccounts = {
  metadata: web3.PublicKey;
  mint: web3.PublicKey;
  mintAuthority: web3.PublicKey;
  payer: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  systemAccount: web3.PublicKey;
};

const createMetadataAccountV2InstructionDiscriminator = [24, 73, 41, 237, 44, 142, 194, 254];

/**
 * Creates a _CreateMetadataAccountV2_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createCreateMetadataAccountV2Instruction(
  accounts: CreateMetadataAccountV2InstructionAccounts,
  args: CreateMetadataAccountV2InstructionArgs,
) {
  const { metadata, mint, mintAuthority, payer, updateAuthority, systemAccount } = accounts;

  const [data] = CreateMetadataAccountV2Struct.serialize({
    instructionDiscriminator: createMetadataAccountV2InstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: mint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: mintAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: payer,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: updateAuthority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: systemAccount,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: web3.SYSVAR_RENT_PUBKEY,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
    keys,
    data,
  });
  return ix;
}
