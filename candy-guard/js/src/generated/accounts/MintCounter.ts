/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link MintCounter}
 * @category Accounts
 * @category generated
 */
export type MintCounterArgs = {
  count: number;
};

export const mintCounterDiscriminator = [29, 59, 15, 69, 46, 22, 227, 173];
/**
 * Holds the data for the {@link MintCounter} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
export class MintCounter implements MintCounterArgs {
  private constructor(readonly count: number) {}

  /**
   * Creates a {@link MintCounter} instance from the provided args.
   */
  static fromArgs(args: MintCounterArgs) {
    return new MintCounter(args.count);
  }

  /**
   * Deserializes the {@link MintCounter} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(accountInfo: web3.AccountInfo<Buffer>, offset = 0): [MintCounter, number] {
    return MintCounter.deserialize(accountInfo.data, offset);
  }

  /**
   * Retrieves the account info from the provided address and deserializes
   * the {@link MintCounter} from its data.
   *
   * @throws Error if no account info is found at the address or if deserialization fails
   */
  static async fromAccountAddress(
    connection: web3.Connection,
    address: web3.PublicKey,
  ): Promise<MintCounter> {
    const accountInfo = await connection.getAccountInfo(address);
    if (accountInfo == null) {
      throw new Error(`Unable to find MintCounter account at ${address}`);
    }
    return MintCounter.fromAccountInfo(accountInfo, 0)[0];
  }

  /**
   * Provides a {@link web3.Connection.getProgramAccounts} config builder,
   * to fetch accounts matching filters that can be specified via that builder.
   *
   * @param programId - the program that owns the accounts we are filtering
   */
  static gpaBuilder(
    programId: web3.PublicKey = new web3.PublicKey('grd1hVewsa8dR1T1JfSFGzQUqgWmc1xXZ3uRRFJJ8XJ'),
  ) {
    return beetSolana.GpaBuilder.fromStruct(programId, mintCounterBeet);
  }

  /**
   * Deserializes the {@link MintCounter} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [MintCounter, number] {
    return mintCounterBeet.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link MintCounter} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return mintCounterBeet.serialize({
      accountDiscriminator: mintCounterDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link MintCounter}
   */
  static get byteSize() {
    return mintCounterBeet.byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link MintCounter} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(MintCounter.byteSize, commitment);
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link MintCounter} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === MintCounter.byteSize;
  }

  /**
   * Returns a readable version of {@link MintCounter} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      count: this.count,
    };
  }
}

/**
 * @category Accounts
 * @category generated
 */
export const mintCounterBeet = new beet.BeetStruct<
  MintCounter,
  MintCounterArgs & {
    accountDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['count', beet.u32],
  ],
  MintCounter.fromArgs,
  'MintCounter',
);