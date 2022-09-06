/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * @category Instructions
 * @category ClaimResource
 * @category generated
 */
export type ClaimResourceInstructionArgs = {
  vaultOwnerBump: number;
};
/**
 * @category Instructions
 * @category ClaimResource
 * @category generated
 */
const claimResourceStruct = new beet.BeetArgsStruct<
  ClaimResourceInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['vaultOwnerBump', beet.u8],
  ],
  'ClaimResourceInstructionArgs',
);
/**
 * Accounts required by the _claimResource_ instruction
 *
 * @property [] market
 * @property [] treasuryHolder
 * @property [] sellingResource
 * @property [**signer**] sellingResourceOwner
 * @property [_writable_] vault
 * @property [_writable_] metadata
 * @property [] owner
 * @property [_writable_] destination
 * @property [] clock
 * @property [] tokenMetadataProgram
 * @category Instructions
 * @category ClaimResource
 * @category generated
 */
export type ClaimResourceInstructionAccounts = {
  market: web3.PublicKey;
  treasuryHolder: web3.PublicKey;
  sellingResource: web3.PublicKey;
  sellingResourceOwner: web3.PublicKey;
  vault: web3.PublicKey;
  metadata: web3.PublicKey;
  owner: web3.PublicKey;
  destination: web3.PublicKey;
  clock: web3.PublicKey;
  tokenMetadataProgram: web3.PublicKey;
};

const claimResourceInstructionDiscriminator = [0, 160, 164, 96, 237, 118, 74, 27];

/**
 * Creates a _ClaimResource_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category ClaimResource
 * @category generated
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
    destination,
    clock,
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
      pubkey: destination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: clock,
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
    programId: new web3.PublicKey('SaLeTjyUa5wXHnGuewUSyJ5JWZaHwz3TxqUntCE9czo'),
    keys,
    data,
  });
  return ix;
}
