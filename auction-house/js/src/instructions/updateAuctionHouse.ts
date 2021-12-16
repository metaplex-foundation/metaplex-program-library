import { AccountMeta, PublicKey, TransactionInstruction } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

export type UpdateAuctionHouseInstructionArgs = {
  sellerFeeBasisPoints: beet.COption<number>;
  requiresSignOff: beet.COption<boolean>;
  canChangeSalePrice: beet.COption<boolean>;
};
const updateAuctionHouseInstructionArgsStruct =
  new beet.BeetArgsStruct<UpdateAuctionHouseInstructionArgs>([
    ['sellerFeeBasisPoints', beet.coption(beet.u16)],
    ['requiresSignOff', beet.coption(beet.bool)],
    ['canChangeSalePrice', beet.coption(beet.bool)],
  ]);
export type UpdateAuctionHouseInstructionAccounts = {
  treasuryMint: PublicKey;
  payer: PublicKey;
  authority: PublicKey;
  newAuthority: PublicKey;
  feeWithdrawalDestination: PublicKey;
  treasuryWithdrawalDestination: PublicKey;
  treasuryWithdrawalDestinationOwner: PublicKey;
  auctionHouse: PublicKey;
  tokenProgram: PublicKey;
  systemProgram: PublicKey;
  ataProgram: PublicKey;
  rent: PublicKey;
};

export function createUpdateAuctionHouseInstruction(
  accounts: UpdateAuctionHouseInstructionAccounts,
  args: UpdateAuctionHouseInstructionArgs,
) {
  const {
    treasuryMint,
    payer,
    authority,
    newAuthority,
    feeWithdrawalDestination,
    treasuryWithdrawalDestination,
    treasuryWithdrawalDestinationOwner,
    auctionHouse,
    tokenProgram,
    systemProgram,
    ataProgram,
    rent,
  } = accounts;

  const [data, _] = updateAuctionHouseInstructionArgsStruct.serialize(args);
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
      isSigner: true,
    },
    {
      pubkey: newAuthority,
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
