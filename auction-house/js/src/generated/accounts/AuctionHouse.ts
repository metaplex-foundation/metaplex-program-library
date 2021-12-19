import { AccountInfo, PublicKey } from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

export type AuctionHouseAccountDataArgs = {
  auctionHouseFeeAccount: PublicKey;
  auctionHouseTreasury: PublicKey;
  treasuryWithdrawalDestination: PublicKey;
  feeWithdrawalDestination: PublicKey;
  treasuryMint: PublicKey;
  authority: PublicKey;
  creator: PublicKey;
  bump: number;
  treasuryBump: number;
  feePayerBump: number;
  sellerFeeBasisPoints: number;
  requiresSignOff: boolean;
  canChangeSalePrice: boolean;
};

export class AuctionHouseAccountData {
  private constructor(
    readonly auctionHouseFeeAccount: PublicKey,
    readonly auctionHouseTreasury: PublicKey,
    readonly treasuryWithdrawalDestination: PublicKey,
    readonly feeWithdrawalDestination: PublicKey,
    readonly treasuryMint: PublicKey,
    readonly authority: PublicKey,
    readonly creator: PublicKey,
    readonly bump: number,
    readonly treasuryBump: number,
    readonly feePayerBump: number,
    readonly sellerFeeBasisPoints: number,
    readonly requiresSignOff: boolean,
    readonly canChangeSalePrice: boolean,
  ) {}

  /**
   * Creates a {@link AuctionHouseAccountData} instance from the provided args.
   */
  static fromArgs(args: AuctionHouseAccountDataArgs) {
    return new AuctionHouseAccountData(
      args.auctionHouseFeeAccount,
      args.auctionHouseTreasury,
      args.treasuryWithdrawalDestination,
      args.feeWithdrawalDestination,
      args.treasuryMint,
      args.authority,
      args.creator,
      args.bump,
      args.treasuryBump,
      args.feePayerBump,
      args.sellerFeeBasisPoints,
      args.requiresSignOff,
      args.canChangeSalePrice,
    );
  }

  /**
   * Deserializes the {@link AuctionHouseAccountData} from the data of the provided {@link AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: AccountInfo<Buffer>,
    offset = 0,
  ): [AuctionHouseAccountData, number] {
    return AuctionHouseAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link AuctionHouseAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [AuctionHouseAccountData, number] {
    return auctionHouseAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link AuctionHouseAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return auctionHouseAccountDataStruct.serialize(this);
  }
}

const auctionHouseAccountDataStruct = new beet.BeetStruct<
  AuctionHouseAccountData,
  AuctionHouseAccountDataArgs
>(
  [
    ['auctionHouseFeeAccount', beetSolana.publicKey],
    ['auctionHouseTreasury', beetSolana.publicKey],
    ['treasuryWithdrawalDestination', beetSolana.publicKey],
    ['feeWithdrawalDestination', beetSolana.publicKey],
    ['treasuryMint', beetSolana.publicKey],
    ['authority', beetSolana.publicKey],
    ['creator', beetSolana.publicKey],
    ['bump', beet.u8],
    ['treasuryBump', beet.u8],
    ['feePayerBump', beet.u8],
    ['sellerFeeBasisPoints', beet.u16],
    ['requiresSignOff', beet.bool],
    ['canChangeSalePrice', beet.bool],
  ],
  AuctionHouseAccountData.fromArgs,
  'AuctionHouseAccountData',
);
