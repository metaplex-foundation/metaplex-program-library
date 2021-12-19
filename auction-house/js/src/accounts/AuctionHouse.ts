// This will wrap the AuctionHouse account to include various checks
// The generated account mainly serves to create/serialize/deserialize

import { strict as assert } from 'assert';
import { AccountInfo, Commitment, Connection, PublicKey } from '@solana/web3.js';
import { AuctionHouseAccountData, AuctionHouseAccountDataArgs } from '../generated/accounts';

export class AuctionHouseAccount {
  constructor(readonly pubkey: PublicKey, readonly data: AuctionHouseAccountData) {}

  static fromAccountInfo(pubkey: PublicKey, info: AccountInfo<Buffer>) {
    assert(
      AuctionHouseAccount.isCompatible(info.data),
      `Data of AccountInfo ${info.data} should be of an AuctionHouseAccount`,
    );
    const [data] = AuctionHouseAccountData.fromAccountInfo(info);
    return new AuctionHouseAccount(pubkey, data);
  }

  static fromAccountArgs(pubkey: PublicKey, args: AuctionHouseAccountDataArgs) {
    const data = AuctionHouseAccountData.fromArgs(args);
    return new AuctionHouseAccount(pubkey, data);
  }

  static isCompatible(buf: Buffer) {
    return AuctionHouseAccountData.isCompatible(buf);
  }

  static async getMinimumBalanceForRentExemption(
    connection: Connection,
    commitment?: Commitment,
  ): Promise<number> {
    return AuctionHouseAccountData.getMinimumBalanceForRentExemption(connection, commitment);
  }

  get pretty() {
    return {
      pubkey: this.pubkey.toBase58(),
      data: this.data.pretty,
    };
  }
}
