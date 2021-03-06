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
 * @category ExecutePartialSale
 * @category generated
 */
export type ExecutePartialSaleInstructionArgs = {
  escrowPaymentBump: number;
  freeTradeStateBump: number;
  programAsSignerBump: number;
  buyerPrice: beet.bignum;
  tokenSize: beet.bignum;
  partialOrderSize: beet.COption<beet.bignum>;
  partialOrderPrice: beet.COption<beet.bignum>;
};
/**
 * @category Instructions
 * @category ExecutePartialSale
 * @category generated
 */
const executePartialSaleStruct = new beet.FixableBeetArgsStruct<
  ExecutePartialSaleInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['escrowPaymentBump', beet.u8],
    ['freeTradeStateBump', beet.u8],
    ['programAsSignerBump', beet.u8],
    ['buyerPrice', beet.u64],
    ['tokenSize', beet.u64],
    ['partialOrderSize', beet.coption(beet.u64)],
    ['partialOrderPrice', beet.coption(beet.u64)],
  ],
  'ExecutePartialSaleInstructionArgs',
);
/**
 * Accounts required by the _executePartialSale_ instruction
 *
 * @property [_writable_] buyer
 * @property [_writable_] seller
 * @property [_writable_] tokenAccount
 * @property [] tokenMint
 * @property [] metadata
 * @property [] treasuryMint
 * @property [_writable_] escrowPaymentAccount
 * @property [_writable_] sellerPaymentReceiptAccount
 * @property [_writable_] buyerReceiptTokenAccount
 * @property [] authority
 * @property [] auctionHouse
 * @property [_writable_] auctionHouseFeeAccount
 * @property [_writable_] auctionHouseTreasury
 * @property [_writable_] buyerTradeState
 * @property [_writable_] sellerTradeState
 * @property [_writable_] freeTradeState
 * @property [] programAsSigner
 * @category Instructions
 * @category ExecutePartialSale
 * @category generated
 */
export type ExecutePartialSaleInstructionAccounts = {
  buyer: web3.PublicKey;
  seller: web3.PublicKey;
  tokenAccount: web3.PublicKey;
  tokenMint: web3.PublicKey;
  metadata: web3.PublicKey;
  treasuryMint: web3.PublicKey;
  escrowPaymentAccount: web3.PublicKey;
  sellerPaymentReceiptAccount: web3.PublicKey;
  buyerReceiptTokenAccount: web3.PublicKey;
  authority: web3.PublicKey;
  auctionHouse: web3.PublicKey;
  auctionHouseFeeAccount: web3.PublicKey;
  auctionHouseTreasury: web3.PublicKey;
  buyerTradeState: web3.PublicKey;
  sellerTradeState: web3.PublicKey;
  freeTradeState: web3.PublicKey;
  programAsSigner: web3.PublicKey;
};

const executePartialSaleInstructionDiscriminator = [163, 18, 35, 157, 49, 164, 203, 133];

/**
 * Creates a _ExecutePartialSale_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category ExecutePartialSale
 * @category generated
 */
export function createExecutePartialSaleInstruction(
  accounts: ExecutePartialSaleInstructionAccounts,
  args: ExecutePartialSaleInstructionArgs,
) {
  const {
    buyer,
    seller,
    tokenAccount,
    tokenMint,
    metadata,
    treasuryMint,
    escrowPaymentAccount,
    sellerPaymentReceiptAccount,
    buyerReceiptTokenAccount,
    authority,
    auctionHouse,
    auctionHouseFeeAccount,
    auctionHouseTreasury,
    buyerTradeState,
    sellerTradeState,
    freeTradeState,
    programAsSigner,
  } = accounts;

  const [data] = executePartialSaleStruct.serialize({
    instructionDiscriminator: executePartialSaleInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: buyer,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: seller,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: tokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: tokenMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: treasuryMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: escrowPaymentAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: sellerPaymentReceiptAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: buyerReceiptTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: authority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: auctionHouse,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: auctionHouseFeeAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: auctionHouseTreasury,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: buyerTradeState,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: sellerTradeState,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: freeTradeState,
      isWritable: true,
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
    {
      pubkey: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: programAsSigner,
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
    programId: new web3.PublicKey('hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk'),
    keys,
    data,
  });
  return ix;
}
