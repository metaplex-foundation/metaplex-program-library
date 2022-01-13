import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type WithdrawFromFeeInstructionArgs = {
  amount: beet.bignum;
};
const withdrawFromFeeStruct = new beet.BeetArgsStruct<
  WithdrawFromFeeInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.fixedSizeArray(beet.u8, 8)],
    ['amount', beet.u64],
  ],
  'WithdrawFromFeeInstructionArgs',
);
export type WithdrawFromFeeInstructionAccounts = {
  authority: web3.PublicKey;
  feeWithdrawalDestination: web3.PublicKey;
  auctionHouseFeeAccount: web3.PublicKey;
  auctionHouse: web3.PublicKey;
};

const withdrawFromFeeInstructionDiscriminator = [179, 208, 190, 154, 32, 179, 19, 59];

export function createWithdrawFromFeeInstruction(
  accounts: WithdrawFromFeeInstructionAccounts,
  args: WithdrawFromFeeInstructionArgs,
) {
  const { authority, feeWithdrawalDestination, auctionHouseFeeAccount, auctionHouse } = accounts;

  const [data] = withdrawFromFeeStruct.serialize({
    instructionDiscriminator: withdrawFromFeeInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
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
      pubkey: web3.SystemProgram.programId,
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
