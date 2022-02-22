import { config, Program } from '@metaplex-foundation/mpl-core';
import { PublicKey } from '@solana/web3.js';
import * as errors from './generated/errors';
import * as instructions from './generated/instructions';
import * as accounts from './generated/accounts';

export class AuctionHouseProgram extends Program {
  static readonly PREFIX = 'auction_house';
  static readonly FEE_PAYER = 'fee_payer';
  static readonly TREASURY = 'treasury';
  static readonly SIGNER = 'signer';

  static readonly PUBKEY = new PublicKey(config.programs.auctionHouse);
  static readonly instructions = instructions;
  static readonly errors = errors;
  static readonly accounts = accounts;

  static readonly AUCTION_HOUSE_PROGRAM_ID = new PublicKey(
    'hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk',
  );

  static async findAuctionHouseAddress(
    creator: PublicKey,
    treasuryMint: PublicKey
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(AuctionHouseProgram.PREFIX, 'utf8'),
        creator.toBuffer(),
        treasuryMint.toBuffer(),
      ],
      AuctionHouseProgram.PUBKEY,
    );
  }

  static async findAuctionHouseProgramAsSignerAddress(): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [Buffer.from(AuctionHouseProgram.PREFIX,), Buffer.from(AuctionHouseProgram.SIGNER)],
      AuctionHouseProgram.AUCTION_HOUSE_PROGRAM_ID,
    );
  }

  static async findAuctionHouseTreasuryAddress(
    auctionHouse: PublicKey,
  ): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [
        Buffer.from(AuctionHouseProgram.PREFIX),
        auctionHouse.toBuffer(),
        Buffer.from(AuctionHouseProgram.TREASURY),
      ],
      AuctionHouseProgram.AUCTION_HOUSE_PROGRAM_ID,
    );
  };

  static async findEscrowPaymentAccountAddress(
    auctionHouse: PublicKey,
    wallet: PublicKey,
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [Buffer.from(AuctionHouseProgram.PREFIX, 'utf8'), auctionHouse.toBuffer(), wallet.toBuffer()],
      AuctionHouseProgram.PUBKEY,
    );
  }

  static async findTradeStateAddress(
    wallet: PublicKey,
    auctionHouse: PublicKey,
    tokenAccount: PublicKey,
    treasuryMint: PublicKey,
    tokenMint: PublicKey,
    price: string,
    tokenSize: string,
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(AuctionHouseProgram.PREFIX, 'utf8'),
        wallet.toBuffer(),
        auctionHouse.toBuffer(),
        tokenAccount.toBuffer(),
        treasuryMint.toBuffer(),
        tokenMint.toBuffer(),
        Buffer.from(price, 'utf8'),
        Buffer.from(tokenSize, 'utf8'),
      ],
      AuctionHouseProgram.PUBKEY,
    );
  }

  static async findAuctionHouseFeeAddress(auctionHouse: PublicKey) {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(AuctionHouseProgram.PREFIX),
        auctionHouse.toBuffer(),
        Buffer.from(AuctionHouseProgram.FEE_PAYER),
      ],
      AuctionHouseProgram.PUBKEY,
    );
  }
}
