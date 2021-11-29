import { ERROR_INVALID_OWNER } from '@metaplex/errors';
import { AnyPublicKey, StringPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { Buffer } from 'buffer';
import { Account } from '../../../Account';
import { AuctionProgram } from '../AuctionProgram';
import { BidderMetadata } from './BidderMetadata';
import { BidderPot } from './BidderPot';

export enum AuctionState {
  Created = 0,
  Started,
  Ended,
}

export enum BidStateType {
  EnglishAuction = 0,
  OpenEdition = 1,
}

export enum PriceFloorType {
  None = 0,
  Minimum = 1,
  BlindedPrice = 2,
}

type BidArgs = { key: StringPublicKey; amount: BN };
export class Bid extends Borsh.Data<BidArgs> {
  static readonly SCHEMA = this.struct([
    ['key', 'pubkeyAsString'],
    ['amount', 'u64'],
  ]);

  key: StringPublicKey;
  amount: BN;
}

type BidStateArgs = { type: BidStateType; bids: Bid[]; max: BN };
export class BidState extends Borsh.Data<BidStateArgs> {
  static readonly SCHEMA = new Map([
    ...Bid.SCHEMA,
    ...this.struct([
      ['type', 'u8'],
      ['bids', [Bid]],
      ['max', 'u64'],
    ]),
  ]);

  type: BidStateType;
  bids: Bid[];
  max: BN;

  getWinnerAt(winnerIndex: number): StringPublicKey | null {
    const convertedIndex = this.bids.length - winnerIndex - 1;

    if (convertedIndex >= 0 && convertedIndex < this.bids.length) {
      return this.bids[convertedIndex].key;
    } else {
      return null;
    }
  }

  getAmountAt(winnerIndex: number): BN | null {
    const convertedIndex = this.bids.length - winnerIndex - 1;

    if (convertedIndex >= 0 && convertedIndex < this.bids.length) {
      return this.bids[convertedIndex].amount;
    } else {
      return null;
    }
  }

  getWinnerIndex(bidder: StringPublicKey): number | null {
    if (!this.bids) return null;

    const index = this.bids.findIndex((b) => b.key === bidder);
    // auction stores data in reverse order
    if (index !== -1) {
      const zeroBased = this.bids.length - index - 1;
      return zeroBased < this.max.toNumber() ? zeroBased : null;
    } else return null;
  }
}

type PriceFloorArgs = { type: PriceFloorType; hash?: Uint8Array; minPrice?: BN };
export class PriceFloor extends Borsh.Data {
  static readonly SCHEMA = this.struct([
    ['type', 'u8'],
    ['hash', [32]],
  ]);

  type: PriceFloorType;
  // It's an array of 32 u8s, when minimum, only first 8 are used (a u64), when blinded price, the entire
  // thing is a hash and not actually a public key, and none is all zeroes
  hash: Uint8Array;
  minPrice?: BN;

  constructor(args: PriceFloorArgs) {
    super();
    this.type = args.type;
    this.hash = args.hash || new Uint8Array(32);
    if (this.type === PriceFloorType.Minimum) {
      if (args.minPrice) {
        this.hash.set(args.minPrice.toArrayLike(Buffer, 'le', 8), 0);
      } else {
        this.minPrice = new BN((args.hash || new Uint8Array(0)).slice(0, 8), 'le');
      }
    }
  }
}

type Args = {
  authority: StringPublicKey;
  tokenMint: StringPublicKey;
  lastBid: BN | null;
  endedAt: BN | null;
  endAuctionAt: BN | null;
  auctionGap: BN | null;
  priceFloor: PriceFloor;
  state: AuctionState;
  bidState: BidState;
  totalUncancelledBids: BN;
};
export class AuctionData extends Borsh.Data<Args> {
  static readonly SCHEMA = new Map([
    ...BidState.SCHEMA,
    ...PriceFloor.SCHEMA,
    ...this.struct([
      ['authority', 'pubkeyAsString'],
      ['tokenMint', 'pubkeyAsString'],
      ['lastBid', { kind: 'option', type: 'u64' }],
      ['endedAt', { kind: 'option', type: 'u64' }],
      ['endAuctionAt', { kind: 'option', type: 'u64' }],
      ['auctionGap', { kind: 'option', type: 'u64' }],
      ['priceFloor', PriceFloor],
      ['state', 'u8'],
      ['bidState', BidState],
    ]),
  ]);

  /// Pubkey of the authority with permission to modify this auction.
  authority: StringPublicKey;
  /// Token mint for the SPL token being used to bid
  tokenMint: StringPublicKey;
  /// The time the last bid was placed, used to keep track of auction timing.
  lastBid: BN | null;
  /// Slot time the auction was officially ended by.
  endedAt: BN | null;
  /// End time is the cut-off point that the auction is forced to end by.
  endAuctionAt: BN | null;
  /// Gap time is the amount of time in slots after the previous bid at which the auction ends.
  auctionGap: BN | null;
  /// Minimum price for any bid to meet.
  priceFloor: PriceFloor;
  /// The state the auction is in, whether it has started or ended.
  state: AuctionState;
  /// Auction Bids, each user may have one bid open at a time.
  bidState: BidState;
  /// Used for precalculation on the front end, not a backend key
  bidRedemptionKey?: StringPublicKey;
}

export class Auction extends Account<AuctionData> {
  static readonly EXTENDED_DATA_SIZE = 8 + 9 + 2 + 200;

  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);

    if (!this.assertOwner(AuctionProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    this.data = AuctionData.deserialize(this.info.data);
  }

  static getPDA(vault: AnyPublicKey) {
    return AuctionProgram.findProgramAddress([
      Buffer.from(AuctionProgram.PREFIX),
      AuctionProgram.PUBKEY.toBuffer(),
      new PublicKey(vault).toBuffer(),
    ]);
  }

  static async findMany(connection: Connection, filters: { authority?: AnyPublicKey } = {}) {
    return (
      await AuctionProgram.getProgramAccounts(connection, {
        filters: [
          // Filter for assigned to authority
          filters.authority && {
            memcmp: {
              offset: 0,
              bytes: new PublicKey(filters.authority).toBase58(),
            },
          },
        ].filter(Boolean),
      })
    )
      .map((account) => {
        try {
          return Auction.from(account);
        } catch (err) {}
      })
      .filter(Boolean);
  }

  async getBidderPots(connection: Connection) {
    return (
      await AuctionProgram.getProgramAccounts(connection, {
        filters: [
          // Filter for BidderPot by data size
          {
            dataSize: BidderPot.DATA_SIZE,
          },
          // Filter for assigned to this auction
          {
            memcmp: {
              offset: 32 + 32,
              bytes: this.pubkey.toBase58(),
            },
          },
        ],
      })
    ).map((account) => BidderPot.from(account));
  }

  async getBidderMetadata(connection: Connection) {
    return (
      await AuctionProgram.getProgramAccounts(connection, {
        filters: [
          // Filter for BidderMetadata by data size
          {
            dataSize: BidderMetadata.DATA_SIZE,
          },
          // Filter for assigned to this auction
          {
            memcmp: {
              offset: 32,
              bytes: this.pubkey.toBase58(),
            },
          },
        ],
      })
    ).map((account) => BidderMetadata.from(account));
  }
}
