import { AccountInfo, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { Account } from '../../../Account';
import { AnyPublicKey, StringPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { AuctionProgram } from '../AuctionProgram';
import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '@metaplex/errors';
import { Buffer } from 'buffer';

type Args = {
  bidderPubkey: StringPublicKey;
  auctionPubkey: StringPublicKey;
  lastBid: BN;
  lastBidTimestamp: BN;
  cancelled: boolean;
};
export class BidderMetadataData extends Borsh.Data<Args> {
  static readonly SCHEMA = this.struct([
    ['bidderPubkey', 'pubkeyAsString'],
    ['auctionPubkey', 'pubkeyAsString'],
    ['lastBid', 'u64'],
    ['lastBidTimestamp', 'u64'],
    ['cancelled', 'u8'],
  ]);

  // Relationship with the bidder who's metadata this covers.
  bidderPubkey: StringPublicKey;
  // Relationship with the auction this bid was placed on.
  auctionPubkey: StringPublicKey;
  // Amount that the user bid.
  lastBid: BN;
  // Tracks the last time this user bid.
  lastBidTimestamp: BN;
  // Whether the last bid the user made was cancelled. This should also be enough to know if the
  // user is a winner, as if cancelled it implies previous bids were also cancelled.
  cancelled: boolean;
}

export class BidderMetadata extends Account<BidderMetadataData> {
  static readonly DATA_SIZE = 32 + 32 + 8 + 8 + 1;

  constructor(key: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(key, info);

    if (!this.assertOwner(AuctionProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (!BidderMetadata.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = BidderMetadataData.deserialize(this.info.data);
  }

  static isCompatible(data: Buffer) {
    return data.length === BidderMetadata.DATA_SIZE;
  }

  static getPDA(auction: AnyPublicKey, bidder: AnyPublicKey) {
    return AuctionProgram.findProgramAddress([
      Buffer.from(AuctionProgram.PREFIX),
      AuctionProgram.PUBKEY.toBuffer(),
      new PublicKey(auction).toBuffer(),
      new PublicKey(bidder).toBuffer(),
      Buffer.from('metadata'),
    ]);
  }
}
