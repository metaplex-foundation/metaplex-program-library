import * as splToken from '@solana/spl-token';
import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type DeprecatedMintPrintingTokensInstructionArgs = {
  mintPrintingTokensViaTokenArgs: definedTypes.MintPrintingTokensViaTokenArgs;
};
const DeprecatedMintPrintingTokensStruct = new beet.BeetArgsStruct<
  DeprecatedMintPrintingTokensInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['mintPrintingTokensViaTokenArgs', definedTypes.mintPrintingTokensViaTokenArgsStruct],
  ],
  'DeprecatedMintPrintingTokensInstructionArgs',
);
export type DeprecatedMintPrintingTokensInstructionAccounts = {
  destination: web3.PublicKey;
  printingMint: web3.PublicKey;
  updateAuthority: web3.PublicKey;
  metadata: web3.PublicKey;
  masterEdition: web3.PublicKey;
};

const deprecatedMintPrintingTokensInstructionDiscriminator = [194, 107, 144, 9, 126, 143, 53, 121];

/**
 * Creates a _DeprecatedMintPrintingTokens_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createDeprecatedMintPrintingTokensInstruction(
  accounts: DeprecatedMintPrintingTokensInstructionAccounts,
  args: DeprecatedMintPrintingTokensInstructionArgs,
) {
  const { destination, printingMint, updateAuthority, metadata, masterEdition } = accounts;

  const [data] = DeprecatedMintPrintingTokensStruct.serialize({
    instructionDiscriminator: deprecatedMintPrintingTokensInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: destination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: printingMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: updateAuthority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: masterEdition,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: splToken.TOKEN_PROGRAM_ID,
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
