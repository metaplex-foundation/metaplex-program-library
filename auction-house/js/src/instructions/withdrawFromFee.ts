import { AccountMeta, PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type WithdrawFromFeeInstructionArgs = {
  amount: beet.bignum;
};
const withdrawFromFeeInstructionArgsStruct =
  new beet.BeetArgsStruct<WithdrawFromFeeInstructionArgs>([['amount', beet.u64]]);
export type WithdrawFromFeeInstructionAccounts = {
  authority: PublicKey;
  feeWithdrawalDestination: PublicKey;
  auctionHouseFeeAccount: PublicKey;
  auctionHouse: PublicKey;
  systemProgram: PublicKey;
};

export function createWithdrawFromFeeInstruction(
  accounts: WithdrawFromFeeInstructionAccounts,
  args: WithdrawFromFeeInstructionArgs,
) {
  const {
    authority,
    feeWithdrawalDestination,
    auctionHouseFeeAccount,
    auctionHouse,
    systemProgram,
  } = accounts;

  const [data, _] = withdrawFromFeeInstructionArgsStruct.serialize(args);
  const keys: AccountMeta[] = [
    {
      pubkey: authority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: feeWithdrawalDestination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: auctionHouseFeeAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: auctionHouse,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: systemProgram,
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
