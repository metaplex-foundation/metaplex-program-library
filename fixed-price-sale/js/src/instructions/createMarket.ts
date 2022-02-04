import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

import { PROGRAM_ID } from '../consts';

export type CreateMarketInstructionArgs = {
  treasuryOwnerBump: number;
  name: string;
  description: string;
  mutable: boolean;
  price: beet.bignum;
  piecesInOneWallet: beet.COption<beet.bignum>;
  startDate: beet.bignum;
  endDate: beet.COption<beet.bignum>;
};
const createMarketStruct = new beet.FixableBeetArgsStruct<
  CreateMarketInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['treasuryOwnerBump', beet.u8],
    ['name', beet.utf8String],
    ['description', beet.utf8String],
    ['mutable', beet.bool],
    ['price', beet.u64],
    ['piecesInOneWallet', beet.coption(beet.u64)],
    ['startDate', beet.u64],
    ['endDate', beet.coption(beet.u64)],
  ],
  'CreateMarketInstructionArgs',
);
export type CreateMarketInstructionAccounts = {
  market: web3.PublicKey;
  store: web3.PublicKey;
  sellingResourceOwner: web3.PublicKey;
  sellingResource: web3.PublicKey;
  mint: web3.PublicKey;
  treasuryHolder: web3.PublicKey;
  owner: web3.PublicKey;
};

const createMarketInstructionDiscriminator = [103, 226, 97, 235, 200, 188, 251, 254];

/**
 * Creates a _CreateMarket_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
export function createCreateMarketInstruction(
  accounts: CreateMarketInstructionAccounts,
  args: CreateMarketInstructionArgs,
) {
  const { market, store, sellingResourceOwner, sellingResource, mint, treasuryHolder, owner } =
    accounts;

  const [data] = createMarketStruct.serialize({
    instructionDiscriminator: createMarketInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: market,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: store,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: sellingResourceOwner,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: sellingResource,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: mint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: treasuryHolder,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: owner,
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
