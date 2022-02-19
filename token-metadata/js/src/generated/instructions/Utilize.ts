import * as splToken from '@solana/spl-token';
import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type UtilizeInstructionArgs = {
  utilizeArgs: definedTypes.UtilizeArgs;
};
const UtilizeStruct = new beet.BeetArgsStruct<
  UtilizeInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['utilizeArgs', definedTypes.utilizeArgsStruct],
  ],
  'UtilizeInstructionArgs',
);
export type UtilizeInstructionAccounts = {
  metadata: web3.PublicKey;
  tokenAccount: web3.PublicKey;
  mint: web3.PublicKey;
  useAuthority: web3.PublicKey;
  owner: web3.PublicKey;
  associatedTokenProgram: web3.PublicKey;
  systemAccount: web3.PublicKey;
  useAuthorityRecord: web3.PublicKey;
  burner: web3.PublicKey;
};

const utilizeInstructionDiscriminator = [104, 146, 242, 209, 176, 174, 185, 163];

/**
 * Creates a _Utilize_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createUtilizeInstruction(
  accounts: UtilizeInstructionAccounts,
  args: UtilizeInstructionArgs,
) {
  const {
    metadata,
    tokenAccount,
    mint,
    useAuthority,
    owner,
    associatedTokenProgram,
    systemAccount,
    useAuthorityRecord,
    burner,
  } = accounts;

  const [data] = UtilizeStruct.serialize({
    instructionDiscriminator: utilizeInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: tokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: mint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: useAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: owner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: associatedTokenProgram,
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
    {
      pubkey: useAuthorityRecord,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: burner,
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
