import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link TradeHistoryAccountData}
 */
export type TradeHistoryAccountDataArgs = {
  market: web3.PublicKey;
  wallet: web3.PublicKey;
  alreadyBought: beet.bignum;
};

const tradeHistoryAccountDiscriminator = [190, 117, 218, 114, 66, 112, 56, 41];
/**
 * Holds the data for the {@link TradeHistoryAccount} and provides de/serialization
 * functionality for that data
 */
export class TradeHistoryAccountData implements TradeHistoryAccountDataArgs {
  private constructor(
    readonly market: web3.PublicKey,
    readonly wallet: web3.PublicKey,
    readonly alreadyBought: beet.bignum,
  ) {}

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link TradeHistoryAccountData}
   */
  static get byteSize() {
    return tradeHistoryAccountDataStruct.byteSize;
  }

  /**
   * Creates a {@link TradeHistoryAccountData} instance from the provided args.
   */
  static fromArgs(args: TradeHistoryAccountDataArgs) {
    return new TradeHistoryAccountData(args.market, args.wallet, args.alreadyBought);
  }

  /**
   * Deserializes the {@link TradeHistoryAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [TradeHistoryAccountData, number] {
    return TradeHistoryAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link TradeHistoryAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [TradeHistoryAccountData, number] {
    return tradeHistoryAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link TradeHistoryAccountData} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      TradeHistoryAccountData.byteSize,
      commitment,
    );
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link TradeHistoryAccountData} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === TradeHistoryAccountData.byteSize;
  }

  /**
   * Serializes the {@link TradeHistoryAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return tradeHistoryAccountDataStruct.serialize({
      accountDiscriminator: tradeHistoryAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns a readable version of {@link TradeHistoryAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      market: this.market.toBase58(),
      wallet: this.wallet.toBase58(),
      alreadyBought: this.alreadyBought,
    };
  }
}

const tradeHistoryAccountDataStruct = new beet.BeetStruct<
  TradeHistoryAccountData,
  TradeHistoryAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['market', beetSolana.publicKey],
    ['wallet', beetSolana.publicKey],
    ['alreadyBought', beet.u64],
  ],
  TradeHistoryAccountData.fromArgs,
  'TradeHistoryAccountData',
);
