import { config, Program } from '@metaplex-foundation/mpl-core';
import { PublicKey } from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';
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
  
  static readonly TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',);
  static readonly SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',);
  static readonly AUCTION_HOUSE_PROGRAM_ID = new PublicKey('hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk',);
  static readonly TOKEN_METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',);

  static async getMetadata (
    mint: anchor.web3.PublicKey,
  ): Promise<anchor.web3.PublicKey> {
    return (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from('metadata'),
          AuctionHouseProgram.TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          mint.toBuffer(),
        ],
        AuctionHouseProgram.TOKEN_METADATA_PROGRAM_ID,
      )
    )[0];
  };

  static async getAuctionHouseTradeState(
    auctionHouse: anchor.web3.PublicKey,
    wallet: anchor.web3.PublicKey,
    tokenAccount: anchor.web3.PublicKey,
    treasuryMint: anchor.web3.PublicKey,
    tokenMint: anchor.web3.PublicKey,
    tokenSize: anchor.BN,
    buyPrice: anchor.BN,
  ): Promise<[PublicKey, number]> {
    return await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(AuctionHouseProgram.PREFIX),
        wallet.toBuffer(),
        auctionHouse.toBuffer(),
        tokenAccount.toBuffer(),
        treasuryMint.toBuffer(),
        tokenMint.toBuffer(),
        buyPrice.toBuffer('le', 8),
        tokenSize.toBuffer('le', 8),
      ],
      AuctionHouseProgram.AUCTION_HOUSE_PROGRAM_ID,
    );
  };

  static async getAtaForMint (
    mint: anchor.web3.PublicKey,
    buyer: anchor.web3.PublicKey,
  ): Promise<[anchor.web3.PublicKey, number]> {
    return await anchor.web3.PublicKey.findProgramAddress(
      [buyer.toBuffer(), AuctionHouseProgram.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      AuctionHouseProgram.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    );
  };

  static async getAuctionHouseProgramAsSigner(
    auctionHouse: PublicKey,
  ): Promise<[PublicKey, number]> {
  return await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(auctionHouse.toString()), Buffer.from('signer')],
    AuctionHouseProgram.AUCTION_HOUSE_PROGRAM_ID,
  );
};


  static async findEscrowPaymentAccount(
    auctionHouse: PublicKey,
    wallet: PublicKey,
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [Buffer.from(AuctionHouseProgram.PREFIX, 'utf8'), auctionHouse.toBuffer(), wallet.toBuffer()],
      AuctionHouseProgram.PUBKEY,
    );
  }

  static async findTradeStateAccount(
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
}