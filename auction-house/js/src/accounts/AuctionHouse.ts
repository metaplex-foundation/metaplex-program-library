import { strict as assert } from 'assert';
import { AccountInfo, Commitment, Connection, PublicKey } from '@solana/web3.js';
import { AuctionHouseAccountData, AuctionHouseAccountDataArgs } from '../generated/accounts';

export class AuctionHouseAccount {
  static hasCorrectByteSize = AuctionHouseAccountData.hasCorrectByteSize;
  static byteSize = AuctionHouseAccountData.byteSize;

  static fromAccountInfo(pubkey: PublicKey, info: AccountInfo<Buffer>) {
    assert(
      AuctionHouseAccount.hasCorrectByteSize(info.data),
      `Data of AccountInfo ${info.data} does not match byte size of AuctionHouseAccount.` +
        `It should be ${AuctionHouseAccountData.byteSize} B, but is ${info.data.byteLength} B`,
    );
    const [data] = AuctionHouseAccountData.fromAccountInfo(info);
    return new AuctionHouseAccount(pubkey, data);
  }

  static fromAccountArgs(pubkey: PublicKey, args: AuctionHouseAccountDataArgs) {
    const data = AuctionHouseAccountData.fromArgs(args);
    return new AuctionHouseAccount(pubkey, data);
  }

  static async getMinimumBalanceForRentExemption(
    connection: Connection,
    commitment?: Commitment,
  ): Promise<number> {
    return AuctionHouseAccountData.getMinimumBalanceForRentExemption(connection, commitment);
  }

  constructor(readonly pubkey: PublicKey, readonly data: AuctionHouseAccountData) {}

  pretty() {
    return {
      pubkey: this.pubkey.toBase58(),
      data: this.data.pretty,
    };
  }
}
