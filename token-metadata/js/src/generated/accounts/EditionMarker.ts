import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * Arguments used to create {@link EditionMarkerAccountData}
 */
export type EditionMarkerAccountDataArgs = {
  key: definedTypes.Key;
  ledger: number[] /* size: 31 */;
};

const editionMarkerAccountDiscriminator = [233, 10, 18, 230, 129, 172, 37, 234];
/**
 * Holds the data for the {@link EditionMarkerAccount} and provides de/serialization
 * functionality for that data
 */
export class EditionMarkerAccountData implements EditionMarkerAccountDataArgs {
  private constructor(readonly key: definedTypes.Key, readonly ledger: number[] /* size: 31 */) {}

  /**
   * Creates a {@link EditionMarkerAccountData} instance from the provided args.
   */
  static fromArgs(args: EditionMarkerAccountDataArgs) {
    return new EditionMarkerAccountData(args.key, args.ledger);
  }

  /**
   * Deserializes the {@link EditionMarkerAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [EditionMarkerAccountData, number] {
    return EditionMarkerAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link EditionMarkerAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [EditionMarkerAccountData, number] {
    return editionMarkerAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link EditionMarkerAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return editionMarkerAccountDataStruct.serialize({
      accountDiscriminator: editionMarkerAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link EditionMarkerAccountData}
   */
  static get byteSize() {
    return editionMarkerAccountDataStruct.byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link EditionMarkerAccountData} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      EditionMarkerAccountData.byteSize,
      commitment,
    );
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link EditionMarkerAccountData} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === EditionMarkerAccountData.byteSize;
  }

  /**
   * Returns a readable version of {@link EditionMarkerAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: this.key,
      ledger: this.ledger,
    };
  }
}

const editionMarkerAccountDataStruct = new beet.BeetStruct<
  EditionMarkerAccountData,
  EditionMarkerAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['key', definedTypes.keyStruct],
    ['ledger', beet.uniformFixedSizeArray(beet.u8, 31)],
  ],
  EditionMarkerAccountData.fromArgs,
  'EditionMarkerAccountData',
);
