import * as splToken from '@solana/spl-token';
import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type DeprecatedCreateMasterEditionInstructionArgs = {
  createMasterEditionArgs: definedTypes.CreateMasterEditionArgs;
};
const DeprecatedCreateMasterEditionStruct = new beet.FixableBeetArgsStruct<
  DeprecatedCreateMasterEditionInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['createMasterEditionArgs', definedTypes.createMasterEditionArgsStruct],
  ],
  'DeprecatedCreateMasterEditionInstructionArgs',
);
export type DeprecatedCreateMasterEditionInstructionAccounts = {
  edition: web3.PublicKey;
  mint: web3.PublicKey;
  printingMint: web3.PublicKey;
  oneTimePrintingAuthorizationMint: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  printingMintAuthority: web3.PublicKey;
  mintAuthority: web3.PublicKey;
  metadata: web3.PublicKey;
  payer: web3.PublicKey;
  systemAccount: web3.PublicKey;
  oneTimePrintingAuthorizationMintAuthority: web3.PublicKey;
};

const deprecatedCreateMasterEditionInstructionDiscriminator = [155, 127, 165, 159, 236, 92, 79, 21];

/**
 * Creates a _DeprecatedCreateMasterEdition_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createDeprecatedCreateMasterEditionInstruction(
  accounts: DeprecatedCreateMasterEditionInstructionAccounts,
  args: DeprecatedCreateMasterEditionInstructionArgs,
) {
  const {
    edition,
    mint,
    printingMint,
    oneTimePrintingAuthorizationMint,
    updateAuthority,
    printingMintAuthority,
    mintAuthority,
    metadata,
    payer,
    systemAccount,
    oneTimePrintingAuthorizationMintAuthority,
  } = accounts;

  const [data] = DeprecatedCreateMasterEditionStruct.serialize({
    instructionDiscriminator: deprecatedCreateMasterEditionInstructionDiscriminator,
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
      pubkey: printingMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: oneTimePrintingAuthorizationMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: updateAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: printingMintAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: mintAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: payer,
      isWritable: false,
      isSigner: true,
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
    {
      pubkey: oneTimePrintingAuthorizationMintAuthority,
      isWritable: false,
      isSigner: true,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
    keys,
    data,
  });
  return ix;
}
