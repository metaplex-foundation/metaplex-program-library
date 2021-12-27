import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

import * as splToken from '@solana/spl-token';

export type DepositInstructionArgs = {
  escrowPaymentBump: number;
  amount: beet.bignum;
};
const depositStruct = new beet.BeetArgsStruct<
  DepositInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.fixedSizeArray(beet.u8, 8)],
    ['escrowPaymentBump', beet.u8],
    ['amount', beet.u64],
  ],
  'DepositInstructionArgs',
);
export type DepositInstructionAccounts = {
  wallet: web3.PublicKey;
  paymentAccount: web3.PublicKey;
  transferAuthority: web3.PublicKey;
  escrowPaymentAccount: web3.PublicKey;
  treasuryMint: web3.PublicKey;
  authority: web3.PublicKey;
  auctionHouse: web3.PublicKey;
  auctionHouseFeeAccount: web3.PublicKey;
};

const depositInstructionDiscriminator = [242, 35, 198, 137, 82, 225, 242, 182];

export function createDepositInstruction(
  accounts: DepositInstructionAccounts,
  args: DepositInstructionArgs,
) {
  const {
    wallet,
    paymentAccount,
    transferAuthority,
    escrowPaymentAccount,
    treasuryMint,
    authority,
    auctionHouse,
    auctionHouseFeeAccount,
  } = accounts;

  const [data] = depositStruct.serialize({
    instructionDiscriminator: depositInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
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
