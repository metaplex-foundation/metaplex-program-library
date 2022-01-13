import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';

import * as splToken from '@solana/spl-token';

export type UpdateAuctionHouseInstructionArgs = {
  sellerFeeBasisPoints: beet.COption<number>;
  requiresSignOff: beet.COption<boolean>;
  canChangeSalePrice: beet.COption<boolean>;
};
const updateAuctionHouseStruct = new beet.BeetArgsStruct<
  UpdateAuctionHouseInstructionArgs & {
    instructionDiscriminator: number[];
  }
>(
  [
    ['instructionDiscriminator', beet.fixedSizeArray(beet.u8, 8)],
    ['sellerFeeBasisPoints', beet.coption(beet.u16)],
    ['requiresSignOff', beet.coption(beet.bool)],
    ['canChangeSalePrice', beet.coption(beet.bool)],
  ],
  'UpdateAuctionHouseInstructionArgs',
);
export type UpdateAuctionHouseInstructionAccounts = {
  treasuryMint: web3.PublicKey;
  payer: web3.PublicKey;
  authority: web3.PublicKey;
  newAuthority: web3.PublicKey;
  feeWithdrawalDestination: web3.PublicKey;
  treasuryWithdrawalDestination: web3.PublicKey;
  treasuryWithdrawalDestinationOwner: web3.PublicKey;
  auctionHouse: web3.PublicKey;
};

const updateAuctionHouseInstructionDiscriminator = [84, 215, 2, 172, 241, 0, 245, 219];

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
  } = accounts;

  const [data] = updateAuctionHouseStruct.serialize({
    instructionDiscriminator: updateAuctionHouseInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
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
