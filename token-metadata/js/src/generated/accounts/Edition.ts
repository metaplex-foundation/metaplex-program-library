import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link EditionAccountData}
 */
export type EditionAccountDataArgs = {
  key: definedTypes.Key;
  parent: web3.PublicKey;
  edition: beet.bignum;
};

const editionAccountDiscriminator = [234, 117, 249, 74, 7, 99, 235, 167];
/**
 * Holds the data for the {@link EditionAccount} and provides de/serialization
 * functionality for that data
 */
export class EditionAccountData implements EditionAccountDataArgs {
  private constructor(
    readonly key: definedTypes.Key,
    readonly parent: web3.PublicKey,
    readonly edition: beet.bignum,
  ) {}

  /**
   * Creates a {@link EditionAccountData} instance from the provided args.
   */
  static fromArgs(args: EditionAccountDataArgs) {
    return new EditionAccountData(args.key, args.parent, args.edition);
  }

  /**
   * Deserializes the {@link EditionAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [EditionAccountData, number] {
    return EditionAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link EditionAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [EditionAccountData, number] {
    return editionAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link EditionAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return editionAccountDataStruct.serialize({
      accountDiscriminator: editionAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link EditionAccountData}
   */
  static get byteSize() {
    return editionAccountDataStruct.byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link EditionAccountData} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(EditionAccountData.byteSize, commitment);
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link EditionAccountData} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === EditionAccountData.byteSize;
  }

  /**
   * Returns a readable version of {@link EditionAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: this.key,
      parent: this.parent.toBase58(),
      edition: this.edition,
    };
  }
}

const editionAccountDataStruct = new beet.BeetStruct<
  EditionAccountData,
  EditionAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['key', definedTypes.keyStruct],
    ['parent', beetSolana.publicKey],
    ['edition', beet.u64],
  ],
  EditionAccountData.fromArgs,
  'EditionAccountData',
);
