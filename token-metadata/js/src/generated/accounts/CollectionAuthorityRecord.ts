import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * Arguments used to create {@link CollectionAuthorityRecordAccountData}
 */
export type CollectionAuthorityRecordAccountDataArgs = {
  key: definedTypes.Key;
  bump: number;
};

const collectionAuthorityRecordAccountDiscriminator = [156, 48, 108, 31, 212, 219, 100, 168];
/**
 * Holds the data for the {@link CollectionAuthorityRecordAccount} and provides de/serialization
 * functionality for that data
 */
export class CollectionAuthorityRecordAccountData
  implements CollectionAuthorityRecordAccountDataArgs
{
  private constructor(readonly key: definedTypes.Key, readonly bump: number) {}

  /**
   * Creates a {@link CollectionAuthorityRecordAccountData} instance from the provided args.
   */
  static fromArgs(args: CollectionAuthorityRecordAccountDataArgs) {
    return new CollectionAuthorityRecordAccountData(args.key, args.bump);
  }

  /**
   * Deserializes the {@link CollectionAuthorityRecordAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [CollectionAuthorityRecordAccountData, number] {
    return CollectionAuthorityRecordAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link CollectionAuthorityRecordAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [CollectionAuthorityRecordAccountData, number] {
    return collectionAuthorityRecordAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link CollectionAuthorityRecordAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return collectionAuthorityRecordAccountDataStruct.serialize({
      accountDiscriminator: collectionAuthorityRecordAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link CollectionAuthorityRecordAccountData}
   */
  static get byteSize() {
    return collectionAuthorityRecordAccountDataStruct.byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link CollectionAuthorityRecordAccountData} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      CollectionAuthorityRecordAccountData.byteSize,
      commitment,
    );
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link CollectionAuthorityRecordAccountData} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === CollectionAuthorityRecordAccountData.byteSize;
  }

  /**
   * Returns a readable version of {@link CollectionAuthorityRecordAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: this.key,
      bump: this.bump,
    };
  }
}

const collectionAuthorityRecordAccountDataStruct = new beet.BeetStruct<
  CollectionAuthorityRecordAccountData,
  CollectionAuthorityRecordAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['key', definedTypes.keyStruct],
    ['bump', beet.u8],
  ],
  CollectionAuthorityRecordAccountData.fromArgs,
  'CollectionAuthorityRecordAccountData',
);
