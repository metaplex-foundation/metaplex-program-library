import { AccountMeta, PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type CreateAuctionHouseInstructionArgs = {
  bump: number;
  feePayerBump: number;
  treasuryBump: number;
  sellerFeeBasisPoints: number;
  requiresSignOff: boolean;
  canChangeSalePrice: boolean;
};
const createAuctionHouseInstructionArgsStruct =
  new beet.BeetArgsStruct<CreateAuctionHouseInstructionArgs>([
    ['bump', beet.u8],
    ['feePayerBump', beet.u8],
    ['treasuryBump', beet.u8],
    ['sellerFeeBasisPoints', beet.u16],
    ['requiresSignOff', beet.bool],
    ['canChangeSalePrice', beet.bool],
  ]);
export type CreateAuctionHouseInstructionAccounts = {
  treasuryMint: PublicKey;
  payer: PublicKey;
  authority: PublicKey;
  feeWithdrawalDestination: PublicKey;
  treasuryWithdrawalDestination: PublicKey;
  treasuryWithdrawalDestinationOwner: PublicKey;
  auctionHouse: PublicKey;
  auctionHouseFeeAccount: PublicKey;
  auctionHouseTreasury: PublicKey;
  tokenProgram: PublicKey;
  systemProgram: PublicKey;
  ataProgram: PublicKey;
  rent: PublicKey;
};

export function createCreateAuctionHouseInstruction(
  accounts: CreateAuctionHouseInstructionAccounts,
  args: CreateAuctionHouseInstructionArgs,
) {
  const {
    treasuryMint,
    payer,
    authority,
    feeWithdrawalDestination,
    treasuryWithdrawalDestination,
    treasuryWithdrawalDestinationOwner,
    auctionHouse,
    auctionHouseFeeAccount,
    auctionHouseTreasury,
    tokenProgram,
    systemProgram,
    ataProgram,
    rent,
  } = accounts;

  const [data, _] = createAuctionHouseInstructionArgsStruct.serialize(args);
  const keys: AccountMeta[] = [
    {
      pubkey: treasuryMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: payer,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: authority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: feeWithdrawalDestination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: treasuryWithdrawalDestination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: treasuryWithdrawalDestinationOwner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: auctionHouse,
      isWritable: true,
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
