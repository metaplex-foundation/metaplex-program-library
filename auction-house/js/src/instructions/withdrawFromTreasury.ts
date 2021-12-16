import { AccountMeta, PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type WithdrawFromTreasuryInstructionArgs = {
  amount: beet.bignum;
};
const withdrawFromTreasuryInstructionArgsStruct =
  new beet.BeetArgsStruct<WithdrawFromTreasuryInstructionArgs>([['amount', beet.u64]]);
export type WithdrawFromTreasuryInstructionAccounts = {
  treasuryMint: PublicKey;
  authority: PublicKey;
  treasuryWithdrawalDestination: PublicKey;
  auctionHouseTreasury: PublicKey;
  auctionHouse: PublicKey;
  tokenProgram: PublicKey;
  systemProgram: PublicKey;
};

export function createWithdrawFromTreasuryInstruction(
  accounts: WithdrawFromTreasuryInstructionAccounts,
  args: WithdrawFromTreasuryInstructionArgs,
) {
  const {
    treasuryMint,
    authority,
    treasuryWithdrawalDestination,
    auctionHouseTreasury,
    auctionHouse,
    tokenProgram,
    systemProgram,
  } = accounts;

  const [data, _] = withdrawFromTreasuryInstructionArgsStruct.serialize(args);
  const keys: AccountMeta[] = [
    {
      pubkey: treasuryMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: authority,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: treasuryWithdrawalDestination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: auctionHouseTreasury,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: auctionHouse,
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
  ];

  const ix = new TransactionInstruction({
    programId: new PublicKey('hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk'),
    keys,
    data,
  });
  return ix;
}
