import * as splToken from '@solana/spl-token';
import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type CreateMasterEditionInstructionArgs = {
  createMasterEditionArgs: definedTypes.CreateMasterEditionArgs;
};
const CreateMasterEditionStruct = new beet.FixableBeetArgsStruct<
  CreateMasterEditionInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['createMasterEditionArgs', definedTypes.createMasterEditionArgsStruct],
  ],
  'CreateMasterEditionInstructionArgs',
);
export type CreateMasterEditionInstructionAccounts = {
  edition: web3.PublicKey;
  mint: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  mintAuthority: web3.PublicKey;
  payer: web3.PublicKey;
  metadata: web3.PublicKey;
  systemAccount: web3.PublicKey;
};

const createMasterEditionInstructionDiscriminator = [179, 210, 96, 96, 57, 25, 79, 69];

/**
 * Creates a _CreateMasterEdition_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createCreateMasterEditionInstruction(
  accounts: CreateMasterEditionInstructionAccounts,
  args: CreateMasterEditionInstructionArgs,
) {
  const { edition, mint, updateAuthority, mintAuthority, payer, metadata, systemAccount } =
    accounts;

  const [data] = CreateMasterEditionStruct.serialize({
    instructionDiscriminator: createMasterEditionInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: edition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: mint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: updateAuthority,
      isWritable: false,
      isSigner: true,
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
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: splToken.TOKEN_PROGRAM_ID,
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
