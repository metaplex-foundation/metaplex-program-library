import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link CreatorAccountData}
 */
export type CreatorAccountDataArgs = {
  address: web3.PublicKey;
  verified: boolean;
  share: number;
};

/**
 * Holds the data for the {@link CreatorAccount} and provides de/serialization
 * functionality for that data
 * ToDo: To replace with Creator from mpl-token-metadata, after it starts to use beet
 */
export class CreatorAccountData implements CreatorAccountDataArgs {
  private constructor(
    readonly address: web3.PublicKey,
    readonly verified: boolean,
    readonly share: number,
  ) {}

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link CreatorAccountData}
   */
  static get byteSize() {
    return creatorAccountDataStruct.byteSize;
  }

  /**
   * Creates a {@link CreatorAccountData} instance from the provided args.
   */
  static fromArgs(args: CreatorAccountDataArgs) {
    return new CreatorAccountData(args.address, args.verified, args.share);
  }

  /**
   * Deserializes the {@link CreatorAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [CreatorAccountData, number] {
    return CreatorAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link CreatorAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [CreatorAccountData, number] {
    return creatorAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link CreatorAccountData} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(CreatorAccountData.byteSize, commitment);
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link CreatorAccountData} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === CreatorAccountData.byteSize;
  }

  /**
   * Serializes the {@link CreatorAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return creatorAccountDataStruct.serialize(this);
  }

  /**
   * Returns a readable version of {@link CreatorAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      address: this.address.toBase58(),
      verified: this.verified,
      share: this.share,
    };
  }
}

export const creatorAccountDataStruct = new beet.BeetStruct<
  CreatorAccountData,
  CreatorAccountDataArgs
>(
  [
    ['address', beetSolana.publicKey],
    ['verified', beet.u8],
    ['share', beet.u8],
  ],
  CreatorAccountData.fromArgs,
  'CreatorAccountData',
);
