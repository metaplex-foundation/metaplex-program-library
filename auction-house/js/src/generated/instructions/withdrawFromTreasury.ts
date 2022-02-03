import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

export type WithdrawFromTreasuryInstructionArgs = {
  amount: beet.bignum;
};
const withdrawFromTreasuryStruct = new beet.BeetArgsStruct<
  WithdrawFromTreasuryInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['amount', beet.u64],
  ],
  'WithdrawFromTreasuryInstructionArgs',
);
export type WithdrawFromTreasuryInstructionAccounts = {
  treasuryMint: web3.PublicKey;
  authority: web3.PublicKey;
  treasuryWithdrawalDestination: web3.PublicKey;
  auctionHouseTreasury: web3.PublicKey;
  auctionHouse: web3.PublicKey;
};

const withdrawFromTreasuryInstructionDiscriminator = [0, 164, 86, 76, 56, 72, 12, 170];

/**
 * Creates a _WithdrawFromTreasury_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
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
  } = accounts;

  const [data] = withdrawFromTreasuryStruct.serialize({
    instructionDiscriminator: withdrawFromTreasuryInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
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
      pubkey: splToken.TOKEN_PROGRAM_ID,
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
    programId: new web3.PublicKey('hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk'),
    keys,
    data,
  });
  return ix;
}
