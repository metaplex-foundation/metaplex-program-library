import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { PROGRAM_ID } from '../consts';

export type ClaimResourceInstructionArgs = {
  vaultOwnerBump: number;
};
const claimResourceStruct = new beet.BeetArgsStruct<
  ClaimResourceInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['vaultOwnerBump', beet.u8],
  ],
  'ClaimResourceInstructionArgs',
);
export type ClaimResourceInstructionAccounts = {
  market: web3.PublicKey;
  treasuryHolder: web3.PublicKey;
  sellingResource: web3.PublicKey;
  sellingResourceOwner: web3.PublicKey;
  vault: web3.PublicKey;
  metadata: web3.PublicKey;
  owner: web3.PublicKey;
  secondaryMetadataCreators: web3.PublicKey;
  destination: web3.PublicKey;
  tokenMetadataProgram: web3.PublicKey;
};

const claimResourceInstructionDiscriminator = [0, 160, 164, 96, 237, 118, 74, 27];

/**
 * Creates a _ClaimResource_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createClaimResourceInstruction(
  accounts: ClaimResourceInstructionAccounts,
  args: ClaimResourceInstructionArgs,
) {
  const {
    market,
    treasuryHolder,
    sellingResource,
    sellingResourceOwner,
    vault,
    metadata,
    owner,
    secondaryMetadataCreators,
    destination,
    tokenMetadataProgram,
  } = accounts;

  const [data] = claimResourceStruct.serialize({
    instructionDiscriminator: claimResourceInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: market,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: treasuryHolder,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: sellingResource,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: sellingResourceOwner,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: vault,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: metadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: owner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: secondaryMetadataCreators,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: destination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: web3.SYSVAR_CLOCK_PUBKEY,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: tokenMetadataProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new web3.TransactionInstruction({
    programId: new web3.PublicKey(PROGRAM_ID),
    keys,
    data,
  });
  return ix;
}
