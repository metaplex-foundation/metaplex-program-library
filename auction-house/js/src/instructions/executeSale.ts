import { AccountMeta, PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type ExecuteSaleInstructionArgs = {
  escrowPaymentBump: number;
  freeTradeStateBump: number;
  programAsSignerBump: number;
  buyerPrice: beet.bignum;
  tokenSize: beet.bignum;
};
const executeSaleInstructionArgsStruct = new beet.BeetArgsStruct<ExecuteSaleInstructionArgs>([
  ['escrowPaymentBump', beet.u8],
  ['freeTradeStateBump', beet.u8],
  ['programAsSignerBump', beet.u8],
  ['buyerPrice', beet.u64],
  ['tokenSize', beet.u64],
]);
export type ExecuteSaleInstructionAccounts = {
  buyer: PublicKey;
  seller: PublicKey;
  tokenAccount: PublicKey;
  tokenMint: PublicKey;
  metadata: PublicKey;
  treasuryMint: PublicKey;
  escrowPaymentAccount: PublicKey;
  sellerPaymentReceiptAccount: PublicKey;
  buyerReceiptTokenAccount: PublicKey;
  authority: PublicKey;
  auctionHouse: PublicKey;
  auctionHouseFeeAccount: PublicKey;
  auctionHouseTreasury: PublicKey;
  buyerTradeState: PublicKey;
  sellerTradeState: PublicKey;
  freeTradeState: PublicKey;
  tokenProgram: PublicKey;
  systemProgram: PublicKey;
  ataProgram: PublicKey;
  programAsSigner: PublicKey;
  rent: PublicKey;
};

export function createExecuteSaleInstruction(
  accounts: ExecuteSaleInstructionAccounts,
  args: ExecuteSaleInstructionArgs,
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
    tokenProgram,
    systemProgram,
    ataProgram,
    programAsSigner,
    rent,
  } = accounts;

  const [data, _] = executeSaleInstructionArgsStruct.serialize(args);
  const keys: AccountMeta[] = [
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
      pubkey: tokenProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: systemProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: ataProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: programAsSigner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: rent,
      isWritable: false,
      isSigner: false,
    },
  ];

  const ix = new TransactionInstruction({
    programId: new PublicKey('hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk'),
    keys,
    data,
  });
  return ix;
}
