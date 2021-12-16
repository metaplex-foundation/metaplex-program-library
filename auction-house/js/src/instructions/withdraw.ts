import { AccountMeta, PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type WithdrawInstructionArgs = {
  escrowPaymentBump: number;
  amount: beet.bignum;
};
const withdrawInstructionArgsStruct = new beet.BeetArgsStruct<WithdrawInstructionArgs>([
  ['escrowPaymentBump', beet.u8],
  ['amount', beet.u64],
]);
export type WithdrawInstructionAccounts = {
  wallet: PublicKey;
  receiptAccount: PublicKey;
  escrowPaymentAccount: PublicKey;
  treasuryMint: PublicKey;
  authority: PublicKey;
  auctionHouse: PublicKey;
  auctionHouseFeeAccount: PublicKey;
  tokenProgram: PublicKey;
  systemProgram: PublicKey;
  ataProgram: PublicKey;
  rent: PublicKey;
};

export function createWithdrawInstruction(
  accounts: WithdrawInstructionAccounts,
  args: WithdrawInstructionArgs,
) {
  const {
    wallet,
    receiptAccount,
    escrowPaymentAccount,
    treasuryMint,
    authority,
    auctionHouse,
    auctionHouseFeeAccount,
    tokenProgram,
    systemProgram,
    ataProgram,
    rent,
  } = accounts;

  const [data, _] = withdrawInstructionArgsStruct.serialize(args);
  const keys: AccountMeta[] = [
    {
      pubkey: wallet,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: receiptAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: escrowPaymentAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: treasuryMint,
      isWritable: false,
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
