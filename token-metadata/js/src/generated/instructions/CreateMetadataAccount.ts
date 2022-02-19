import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type CreateMetadataAccountInstructionArgs = {
  createMetadataAccountArgs: definedTypes.CreateMetadataAccountArgs;
};
const CreateMetadataAccountStruct = new beet.BeetArgsStruct<
  CreateMetadataAccountInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['createMetadataAccountArgs', definedTypes.createMetadataAccountArgsStruct],
  ],
  'CreateMetadataAccountInstructionArgs',
);
export type CreateMetadataAccountInstructionAccounts = {
  metadata: web3.PublicKey;
  mint: web3.PublicKey;
  mintAuthority: web3.PublicKey;
  payer: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  systemAccount: web3.PublicKey;
};

const createMetadataAccountInstructionDiscriminator = [75, 73, 45, 178, 212, 194, 127, 113];

/**
 * Creates a _CreateMetadataAccount_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createCreateMetadataAccountInstruction(
  accounts: CreateMetadataAccountInstructionAccounts,
  args: CreateMetadataAccountInstructionArgs,
) {
  const { metadata, mint, mintAuthority, payer, updateAuthority, systemAccount } = accounts;

  const [data] = CreateMetadataAccountStruct.serialize({
    instructionDiscriminator: createMetadataAccountInstructionDiscriminator,
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
