import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { CreatorAccountData, creatorAccountDataStruct } from '.';

/**
 * Arguments used to create {@link SecondaryMetadataCreatorsAccountData}
 */
export type SecondaryMetadataCreatorsAccountDataArgs = {
  creators: CreatorAccountData[];
};

const secondaryMetadataCreatorsAccountDiscriminator = [178, 90, 101, 55, 140, 245, 196, 33];
/**
 * Holds the data for the {@link SecondaryMetadataCreatorsAccount} and provides de/serialization
 * functionality for that data
 */
export class SecondaryMetadataCreatorsAccountData
  implements SecondaryMetadataCreatorsAccountDataArgs
{
  private constructor(readonly creators: CreatorAccountData[]) {}

  /**
   * Creates a {@link SecondaryMetadataCreatorsAccountData} instance from the provided args.
   */
  static fromArgs(args: SecondaryMetadataCreatorsAccountDataArgs) {
    return new SecondaryMetadataCreatorsAccountData(args.creators);
  }

  /**
   * Deserializes the {@link SecondaryMetadataCreatorsAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [SecondaryMetadataCreatorsAccountData, number] {
    return SecondaryMetadataCreatorsAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link SecondaryMetadataCreatorsAccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: SecondaryMetadataCreatorsAccountDataArgs) {
    const instance = SecondaryMetadataCreatorsAccountData.fromArgs(args);
    return secondaryMetadataCreatorsAccountDataStruct.toFixedFromValue({
      accountDiscriminator: secondaryMetadataCreatorsAccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link SecondaryMetadataCreatorsAccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: SecondaryMetadataCreatorsAccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      SecondaryMetadataCreatorsAccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Deserializes the {@link SecondaryMetadataCreatorsAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [SecondaryMetadataCreatorsAccountData, number] {
    return secondaryMetadataCreatorsAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link SecondaryMetadataCreatorsAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return secondaryMetadataCreatorsAccountDataStruct.serialize({
      accountDiscriminator: secondaryMetadataCreatorsAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns a readable version of {@link SecondaryMetadataCreatorsAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      creators: this.creators,
    };
  }
}

const secondaryMetadataCreatorsAccountDataStruct = new beet.FixableBeetStruct<
  SecondaryMetadataCreatorsAccountData,
  SecondaryMetadataCreatorsAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['creators', beet.array(creatorAccountDataStruct)],
  ],
  SecondaryMetadataCreatorsAccountData.fromArgs,
  'SecondaryMetadataCreatorsAccountData',
);
