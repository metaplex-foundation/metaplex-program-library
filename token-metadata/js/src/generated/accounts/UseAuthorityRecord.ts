import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * Arguments used to create {@link UseAuthorityRecordAccountData}
 */
export type UseAuthorityRecordAccountDataArgs = {
  key: definedTypes.Key;
  allowedUses: beet.bignum;
  bump: number;
};

const useAuthorityRecordAccountDiscriminator = [227, 200, 230, 197, 244, 198, 172, 50];
/**
 * Holds the data for the {@link UseAuthorityRecordAccount} and provides de/serialization
 * functionality for that data
 */
export class UseAuthorityRecordAccountData implements UseAuthorityRecordAccountDataArgs {
  private constructor(
    readonly key: definedTypes.Key,
    readonly allowedUses: beet.bignum,
    readonly bump: number,
  ) {}

  /**
   * Creates a {@link UseAuthorityRecordAccountData} instance from the provided args.
   */
  static fromArgs(args: UseAuthorityRecordAccountDataArgs) {
    return new UseAuthorityRecordAccountData(args.key, args.allowedUses, args.bump);
  }

  /**
   * Deserializes the {@link UseAuthorityRecordAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [UseAuthorityRecordAccountData, number] {
    return UseAuthorityRecordAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link UseAuthorityRecordAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [UseAuthorityRecordAccountData, number] {
    return useAuthorityRecordAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link UseAuthorityRecordAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return useAuthorityRecordAccountDataStruct.serialize({
      accountDiscriminator: useAuthorityRecordAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link UseAuthorityRecordAccountData}
   */
  static get byteSize() {
    return useAuthorityRecordAccountDataStruct.byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link UseAuthorityRecordAccountData} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      UseAuthorityRecordAccountData.byteSize,
      commitment,
    );
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link UseAuthorityRecordAccountData} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === UseAuthorityRecordAccountData.byteSize;
  }

  /**
   * Returns a readable version of {@link UseAuthorityRecordAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: this.key,
      allowedUses: this.allowedUses,
      bump: this.bump,
    };
  }
}

const useAuthorityRecordAccountDataStruct = new beet.BeetStruct<
  UseAuthorityRecordAccountData,
  UseAuthorityRecordAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['key', definedTypes.keyStruct],
    ['allowedUses', beet.u64],
    ['bump', beet.u8],
  ],
  UseAuthorityRecordAccountData.fromArgs,
  'UseAuthorityRecordAccountData',
);
