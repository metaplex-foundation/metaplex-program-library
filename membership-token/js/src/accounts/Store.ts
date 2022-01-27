import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';
import * as beet from '@metaplex-foundation/beet';

/**
 * Arguments used to create {@link StoreAccountData}
 */
export type StoreAccountDataArgs = {
  admin: web3.PublicKey;
  name: string;
  description: string;
};

const storeAccountDiscriminator = [130, 48, 247, 244, 182, 191, 30, 26];
/**
 * Holds the data for the {@link StoreAccount} and provides de/serialization
 * functionality for that data
 */
export class StoreAccountData implements StoreAccountDataArgs {
  private constructor(
    readonly admin: web3.PublicKey,
    readonly name: string,
    readonly description: string,
  ) {}

  /**
   * Creates a {@link StoreAccountData} instance from the provided args.
   */
  static fromArgs(args: StoreAccountDataArgs) {
    return new StoreAccountData(args.admin, args.name, args.description);
  }

  /**
   * Deserializes the {@link StoreAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [StoreAccountData, number] {
    return StoreAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link StoreAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [StoreAccountData, number] {
    return storeAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link StoreAccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: StoreAccountDataArgs) {
    const instance = StoreAccountData.fromArgs(args);
    return storeAccountDataStruct.toFixedFromValue({
      accountDiscriminator: storeAccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link StoreAccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: StoreAccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      StoreAccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Serializes the {@link StoreAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return storeAccountDataStruct.serialize({
      accountDiscriminator: storeAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns a readable version of {@link StoreAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      admin: this.admin.toBase58(),
      name: this.name,
      description: this.description,
    };
  }
}

const storeAccountDataStruct = new beet.FixableBeetStruct<
  StoreAccountData,
  StoreAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['admin', beetSolana.publicKey],
    ['name', beet.utf8String],
    ['description', beet.utf8String],
  ],
  StoreAccountData.fromArgs,
  'StoreAccountData',
);
