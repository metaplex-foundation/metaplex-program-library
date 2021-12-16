import { AccountMeta, PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type BuyInstructionArgs = {
  tradeStateBump: number;
  escrowPaymentBump: number;
  buyerPrice: beet.bignum;
  tokenSize: beet.bignum;
};
const buyInstructionArgsStruct = new beet.BeetArgsStruct<BuyInstructionArgs>([
  ['tradeStateBump', beet.u8],
  ['escrowPaymentBump', beet.u8],
  ['buyerPrice', beet.u64],
  ['tokenSize', beet.u64],
]);
export type BuyInstructionAccounts = {
  wallet: PublicKey;
  paymentAccount: PublicKey;
  transferAuthority: PublicKey;
  treasuryMint: PublicKey;
  tokenAccount: PublicKey;
  metadata: PublicKey;
  escrowPaymentAccount: PublicKey;
  authority: PublicKey;
  auctionHouse: PublicKey;
  auctionHouseFeeAccount: PublicKey;
  buyerTradeState: PublicKey;
  tokenProgram: PublicKey;
  systemProgram: PublicKey;
  rent: PublicKey;
};

export function createBuyInstruction(accounts: BuyInstructionAccounts, args: BuyInstructionArgs) {
  const {
    wallet,
    paymentAccount,
    transferAuthority,
    treasuryMint,
    tokenAccount,
    metadata,
    escrowPaymentAccount,
    authority,
    auctionHouse,
    auctionHouseFeeAccount,
    buyerTradeState,
    tokenProgram,
    systemProgram,
    rent,
  } = accounts;

  const [data, _] = buyInstructionArgsStruct.serialize(args);
  const keys: AccountMeta[] = [
    {
      pubkey: wallet,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: paymentAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: transferAuthority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: treasuryMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: tokenAccount,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: escrowPaymentAccount,
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
      pubkey: buyerTradeState,
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
