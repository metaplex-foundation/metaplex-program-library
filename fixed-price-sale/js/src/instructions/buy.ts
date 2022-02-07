import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

import { PROGRAM_ID } from '../consts';

export type BuyInstructionArgs = {
  tradeHistoryBump: number;
  vaultOwnerBump: number;
};
const buyStruct = new beet.BeetArgsStruct<
  BuyInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['tradeHistoryBump', beet.u8],
    ['vaultOwnerBump', beet.u8],
  ],
  'BuyInstructionArgs',
);
export type BuyInstructionAccounts = {
  market: web3.PublicKey;
  sellingResource: web3.PublicKey;
  userTokenAccount: web3.PublicKey;
  userWallet: web3.PublicKey;
  tradeHistory: web3.PublicKey;
  treasuryHolder: web3.PublicKey;
  newMetadata: web3.PublicKey;
  newEdition: web3.PublicKey;
  masterEdition: web3.PublicKey;
  newMint: web3.PublicKey;
  editionMarker: web3.PublicKey;
  vault: web3.PublicKey;
  owner: web3.PublicKey;
  masterEditionMetadata: web3.PublicKey;
  clock: web3.PublicKey;
  tokenMetadataProgram: web3.PublicKey;
};

const buyInstructionDiscriminator = [102, 6, 61, 18, 1, 218, 235, 234];

/**
 * Creates a _Buy_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createBuyInstruction(accounts: BuyInstructionAccounts, args: BuyInstructionArgs) {
  const {
    market,
    sellingResource,
    userTokenAccount,
    userWallet,
    tradeHistory,
    treasuryHolder,
    newMetadata,
    newEdition,
    masterEdition,
    newMint,
    editionMarker,
    vault,
    owner,
    masterEditionMetadata,
    clock,
    tokenMetadataProgram,
  } = accounts;

  const [data] = buyStruct.serialize({
    instructionDiscriminator: buyInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: market,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: sellingResource,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: userTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: userWallet,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: tradeHistory,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: treasuryHolder,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: newMetadata,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: newEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: masterEdition,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: newMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: editionMarker,
      isWritable: true,
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
      pubkey: masterEditionMetadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: clock,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: web3.SYSVAR_RENT_PUBKEY,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: tokenMetadataProgram,
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
