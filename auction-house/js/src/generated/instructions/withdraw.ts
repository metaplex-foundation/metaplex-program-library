import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

import * as splToken from '@solana/spl-token';

export type WithdrawInstructionArgs = {
  escrowPaymentBump: number;
  amount: beet.bignum;
};
const withdrawStruct = new beet.BeetArgsStruct<
  WithdrawInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.fixedSizeArray(beet.u8, 8)],
    ['escrowPaymentBump', beet.u8],
    ['amount', beet.u64],
  ],
  'WithdrawInstructionArgs',
);
export type WithdrawInstructionAccounts = {
  wallet: web3.PublicKey;
  receiptAccount: web3.PublicKey;
  escrowPaymentAccount: web3.PublicKey;
  treasuryMint: web3.PublicKey;
  authority: web3.PublicKey;
  auctionHouse: web3.PublicKey;
  auctionHouseFeeAccount: web3.PublicKey;
};

const withdrawInstructionDiscriminator = [183, 18, 70, 156, 148, 109, 161, 34];

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
  } = accounts;

  const [data] = withdrawStruct.serialize({
    instructionDiscriminator: withdrawInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
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
