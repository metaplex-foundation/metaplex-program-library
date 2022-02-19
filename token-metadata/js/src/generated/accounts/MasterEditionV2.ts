import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * Arguments used to create {@link MasterEditionV2AccountData}
 */
export type MasterEditionV2AccountDataArgs = {
  key: definedTypes.Key;
  supply: beet.bignum;
  maxSupply: beet.COption<beet.bignum>;
};

const masterEditionV2AccountDiscriminator = [101, 59, 163, 207, 238, 16, 170, 159];
/**
 * Holds the data for the {@link MasterEditionV2Account} and provides de/serialization
 * functionality for that data
 */
export class MasterEditionV2AccountData implements MasterEditionV2AccountDataArgs {
  private constructor(
    readonly key: definedTypes.Key,
    readonly supply: beet.bignum,
    readonly maxSupply: beet.COption<beet.bignum>,
  ) {}

  /**
   * Creates a {@link MasterEditionV2AccountData} instance from the provided args.
   */
  static fromArgs(args: MasterEditionV2AccountDataArgs) {
    return new MasterEditionV2AccountData(args.key, args.supply, args.maxSupply);
  }

  /**
   * Deserializes the {@link MasterEditionV2AccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [MasterEditionV2AccountData, number] {
    return MasterEditionV2AccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link MasterEditionV2AccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [MasterEditionV2AccountData, number] {
    return masterEditionV2AccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link MasterEditionV2AccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return masterEditionV2AccountDataStruct.serialize({
      accountDiscriminator: masterEditionV2AccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link MasterEditionV2AccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: MasterEditionV2AccountDataArgs) {
    const instance = MasterEditionV2AccountData.fromArgs(args);
    return masterEditionV2AccountDataStruct.toFixedFromValue({
      accountDiscriminator: masterEditionV2AccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link MasterEditionV2AccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: MasterEditionV2AccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      MasterEditionV2AccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Returns a readable version of {@link MasterEditionV2AccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: this.key,
      supply: this.supply,
      maxSupply: this.maxSupply,
    };
  }
}

const masterEditionV2AccountDataStruct = new beet.FixableBeetStruct<
  MasterEditionV2AccountData,
  MasterEditionV2AccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['key', definedTypes.keyStruct],
    ['supply', beet.u64],
    ['maxSupply', beet.coption(beet.u64)],
  ],
  MasterEditionV2AccountData.fromArgs,
  'MasterEditionV2AccountData',
);
