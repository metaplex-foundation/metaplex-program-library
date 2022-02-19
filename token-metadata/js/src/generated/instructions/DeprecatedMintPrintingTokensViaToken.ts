import * as splToken from '@solana/spl-token';
import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type DeprecatedMintPrintingTokensViaTokenInstructionArgs = {
  mintPrintingTokensViaTokenArgs: definedTypes.MintPrintingTokensViaTokenArgs;
};
const DeprecatedMintPrintingTokensViaTokenStruct = new beet.BeetArgsStruct<
  DeprecatedMintPrintingTokensViaTokenInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['mintPrintingTokensViaTokenArgs', definedTypes.mintPrintingTokensViaTokenArgsStruct],
  ],
  'DeprecatedMintPrintingTokensViaTokenInstructionArgs',
);
export type DeprecatedMintPrintingTokensViaTokenInstructionAccounts = {
  destination: web3.PublicKey;
  token: web3.PublicKey;
  oneTimePrintingAuthorizationMint: web3.PublicKey;
  printingMint: web3.PublicKey;
  burnAuthority: web3.PublicKey;
  metadata: web3.PublicKey;
  masterEdition: web3.PublicKey;
};

const deprecatedMintPrintingTokensViaTokenInstructionDiscriminator = [
  84, 34, 152, 133, 145, 48, 4, 223,
];

/**
 * Creates a _DeprecatedMintPrintingTokensViaToken_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createDeprecatedMintPrintingTokensViaTokenInstruction(
  accounts: DeprecatedMintPrintingTokensViaTokenInstructionAccounts,
  args: DeprecatedMintPrintingTokensViaTokenInstructionArgs,
) {
  const {
    destination,
    token,
    oneTimePrintingAuthorizationMint,
    printingMint,
    burnAuthority,
    metadata,
    masterEdition,
  } = accounts;

  const [data] = DeprecatedMintPrintingTokensViaTokenStruct.serialize({
    instructionDiscriminator: deprecatedMintPrintingTokensViaTokenInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: destination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: token,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: oneTimePrintingAuthorizationMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: printingMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: burnAuthority,
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
