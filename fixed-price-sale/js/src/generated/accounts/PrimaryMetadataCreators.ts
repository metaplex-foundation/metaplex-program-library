import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { CreatorAccountData, creatorAccountDataStruct } from '.';

/**
 * Arguments used to create {@link PrimaryMetadataCreatorsAccountData}
 */
export type PrimaryMetadataCreatorsAccountDataArgs = {
  creators: CreatorAccountData[];
};

const primaryMetadataCreatorsAccountDiscriminator = [66, 131, 48, 36, 100, 130, 177, 11];
/**
 * Holds the data for the {@link PrimaryMetadataCreatorsAccount} and provides de/serialization
 * functionality for that data
 */
export class PrimaryMetadataCreatorsAccountData implements PrimaryMetadataCreatorsAccountDataArgs {
  private constructor(readonly creators: CreatorAccountData[]) {}

  /**
   * Creates a {@link PrimaryMetadataCreatorsAccountData} instance from the provided args.
   */
  static fromArgs(args: PrimaryMetadataCreatorsAccountDataArgs) {
    return new PrimaryMetadataCreatorsAccountData(args.creators);
  }

  /**
   * Deserializes the {@link PrimaryMetadataCreatorsAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [PrimaryMetadataCreatorsAccountData, number] {
    return PrimaryMetadataCreatorsAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link PrimaryMetadataCreatorsAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [PrimaryMetadataCreatorsAccountData, number] {
    return primaryMetadataCreatorsAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link PrimaryMetadataCreatorsAccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: PrimaryMetadataCreatorsAccountDataArgs) {
    const instance = PrimaryMetadataCreatorsAccountData.fromArgs(args);
    return primaryMetadataCreatorsAccountDataStruct.toFixedFromValue({
      accountDiscriminator: primaryMetadataCreatorsAccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link PrimaryMetadataCreatorsAccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: PrimaryMetadataCreatorsAccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      PrimaryMetadataCreatorsAccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Serializes the {@link PrimaryMetadataCreatorsAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return primaryMetadataCreatorsAccountDataStruct.serialize({
      accountDiscriminator: primaryMetadataCreatorsAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns a readable version of {@link PrimaryMetadataCreatorsAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      creators: this.creators,
    };
  }
}

const primaryMetadataCreatorsAccountDataStruct = new beet.FixableBeetStruct<
  PrimaryMetadataCreatorsAccountData,
  PrimaryMetadataCreatorsAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['creators', beet.array(creatorAccountDataStruct)],
  ],
  PrimaryMetadataCreatorsAccountData.fromArgs,
  'PrimaryMetadataCreatorsAccountData',
);
