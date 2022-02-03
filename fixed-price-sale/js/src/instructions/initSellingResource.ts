import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as splToken from '@solana/spl-token';

import { PROGRAM_ID } from '../consts';

export type InitSellingResourceInstructionArgs = {
  masterEditionBump: number;
  vaultOwnerBump: number;
  maxSupply: beet.COption<beet.bignum>;
};
const initSellingResourceStruct = new beet.FixableBeetArgsStruct<
  InitSellingResourceInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['masterEditionBump', beet.u8],
    ['vaultOwnerBump', beet.u8],
    ['maxSupply', beet.coption(beet.u64)],
  ],
  'InitSellingResourceInstructionArgs',
);
export type InitSellingResourceInstructionAccounts = {
  store: web3.PublicKey;
  admin: web3.PublicKey;
  sellingResource: web3.PublicKey;
  sellingResourceOwner: web3.PublicKey;
  resourceMint: web3.PublicKey;
  masterEdition: web3.PublicKey;
  metadata: web3.PublicKey;
  vault: web3.PublicKey;
  owner: web3.PublicKey;
  resourceToken: web3.PublicKey;
};

const initSellingResourceInstructionDiscriminator = [56, 15, 222, 211, 147, 205, 4, 145];

/**
 * Creates a _InitSellingResource_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createInitSellingResourceInstruction(
  accounts: InitSellingResourceInstructionAccounts,
  args: InitSellingResourceInstructionArgs,
) {
  const {
    store,
    admin,
    sellingResource,
    sellingResourceOwner,
    resourceMint,
    masterEdition,
    metadata,
    vault,
    owner,
    resourceToken,
  } = accounts;

  const [data] = initSellingResourceStruct.serialize({
    instructionDiscriminator: initSellingResourceInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: store,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: admin,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: sellingResource,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: sellingResourceOwner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: resourceMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: masterEdition,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: vault,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: owner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: resourceToken,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: web3.SYSVAR_RENT_PUBKEY,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: splToken.TOKEN_PROGRAM_ID,
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
