import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import bs58 from 'bs58';
import { AnyPublicKey, StringPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { Account } from '../../../Account';
import { BidRedemptionTicket, WINNER_INDEX_OFFSETS } from './BidRedemptionTicket';
import { MetaplexKey, MetaplexProgram } from '../MetaplexProgram';
import {
  ERROR_DEPRECATED_ACCOUNT_DATA,
  ERROR_INVALID_ACCOUNT_DATA,
  ERROR_INVALID_OWNER,
} from '@metaplex/errors';
import { Auction } from '../../auction';
import { Buffer } from 'buffer';

export enum AuctionManagerStatus {
  Initialized,
  Validated,
  Running,
  Disbursing,
  Finished,
}

export class AuctionManagerStateV2 extends Borsh.Data<{
  status: AuctionManagerStatus;
  safetyConfigItemsValidated: BN;
  bidsPushedToAcceptPayment: BN;
  hasParticipation: boolean;
}> {
  static readonly SCHEMA = this.struct([
    ['status', 'u8'],
    ['safetyConfigItemsValidated', 'u64'],
    ['bidsPushedToAcceptPayment', 'u64'],
    ['hasParticipation', 'u8'],
  ]);

  status: AuctionManagerStatus = AuctionManagerStatus.Initialized;
  safetyConfigItemsValidated: BN = new BN(0);
  bidsPushedToAcceptPayment: BN = new BN(0);
  hasParticipation = false;
}

type Args = {
  store: StringPublicKey;
  authority: StringPublicKey;
  auction: StringPublicKey;
  vault: StringPublicKey;
  acceptPayment: StringPublicKey;
  state: AuctionManagerStateV2;
};
export class AuctionManagerV2Data extends Borsh.Data<Args> {
  static readonly SCHEMA = new Map([
    ...AuctionManagerStateV2.SCHEMA,
    ...this.struct([
      ['key', 'u8'],
      ['store', 'pubkeyAsString'],
      ['authority', 'pubkeyAsString'],
      ['auction', 'pubkeyAsString'],
      ['vault', 'pubkeyAsString'],
      ['acceptPayment', 'pubkeyAsString'],
      ['state', AuctionManagerStateV2],
    ]),
  ]);

  key: MetaplexKey;
  store: StringPublicKey;
  authority: StringPublicKey;
  auction: StringPublicKey;
  vault: StringPublicKey;
  acceptPayment: StringPublicKey;
  state: AuctionManagerStateV2;

  constructor(args: Args) {
    super(args);
    this.key = MetaplexKey.AuctionManagerV2;
  }
}

export class AuctionManager extends Account<AuctionManagerV2Data> {
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);

    if (!this.assertOwner(MetaplexProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (AuctionManager.isAuctionManagerV1(this.info.data)) {
      throw ERROR_DEPRECATED_ACCOUNT_DATA();
    } else if (AuctionManager.isAuctionManagerV2(this.info.data)) {
      this.data = AuctionManagerV2Data.deserialize(this.info.data);
    } else {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }
  }

  static isCompatible(data: Buffer) {
    return AuctionManager.isAuctionManagerV1(data) || AuctionManager.isAuctionManagerV2(data);
  }

  static isAuctionManagerV1(data: Buffer) {
    return data[0] === MetaplexKey.AuctionManagerV1;
  }

  static isAuctionManagerV2(data: Buffer) {
    return data[0] === MetaplexKey.AuctionManagerV2;
  }

  static getPDA(auction: AnyPublicKey) {
    return MetaplexProgram.findProgramAddress([
      Buffer.from(MetaplexProgram.PREFIX),
      new PublicKey(auction).toBuffer(),
    ]);
  }

  static async findMany(
    connection: Connection,
    filters: { store?: AnyPublicKey; authority?: AnyPublicKey } = {},
  ) {
    return (
      await MetaplexProgram.getProgramAccounts(connection, {
        filters: [
          // Filter for AuctionManagerV2 by key
          {
            memcmp: {
              offset: 0,
              bytes: bs58.encode(Buffer.from([MetaplexKey.AuctionManagerV2])),
            },
          },
          // Filter for assigned to store
          filters.store && {
            memcmp: {
              offset: 1,
              bytes: new PublicKey(filters.store).toBase58(),
            },
          },
          // Filter for assigned to authority
          filters.authority && {
            memcmp: {
              offset: 33,
              bytes: new PublicKey(filters.authority).toBase58(),
            },
          },
        ].filter(Boolean),
      })
    ).map((account) => AuctionManager.from(account));
  }

  async getAuction(connection: Connection) {
    return Auction.load(connection, this.data.auction);
  }

  async getBidRedemptionTickets(connection: Connection, haveWinnerIndex = true) {
    return (
      await MetaplexProgram.getProgramAccounts(connection, {
        filters: [
          // Filter for BidRedemptionTicketV2 by key
          {
            memcmp: {
              offset: 0,
              bytes: bs58.encode(Buffer.from([MetaplexKey.BidRedemptionTicketV2])),
            },
          },
          // Filter for assigned to this auction manager
          {
            memcmp: {
              offset: WINNER_INDEX_OFFSETS[+haveWinnerIndex],
              bytes: this.pubkey.toBase58(),
            },
          },
        ],
      })
    ).map((account) => BidRedemptionTicket.from(account));
  }
}
