/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { Key, keyBeet } from '../types/Key';

/**
 * Arguments used to create {@link MasterEditionV2}
 * @category Accounts
 * @category generated
 */
export type MasterEditionV2Args = {
  key: Key;
  supply: beet.bignum;
  maxSupply: beet.COption<beet.bignum>;
};
/**
 * Holds the data for the {@link MasterEditionV2} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
export class MasterEditionV2 implements MasterEditionV2Args {
  private constructor(
    readonly key: Key,
    readonly supply: beet.bignum,
    readonly maxSupply: beet.COption<beet.bignum>,
  ) {}

  /**
   * Creates a {@link MasterEditionV2} instance from the provided args.
   */
  static fromArgs(args: MasterEditionV2Args) {
    return new MasterEditionV2(args.key, args.supply, args.maxSupply);
  }

  /**
   * Deserializes the {@link MasterEditionV2} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [MasterEditionV2, number] {
    return MasterEditionV2.deserialize(accountInfo.data, offset);
  }

  /**
   * Retrieves the account info from the provided address and deserializes
   * the {@link MasterEditionV2} from its data.
   *
   * @throws Error if no account info is found at the address or if deserialization fails
   */
  static async fromAccountAddress(
    connection: web3.Connection,
    address: web3.PublicKey,
  ): Promise<MasterEditionV2> {
    const accountInfo = await connection.getAccountInfo(address);
    if (accountInfo == null) {
      throw new Error(`Unable to find MasterEditionV2 account at ${address}`);
    }
    return MasterEditionV2.fromAccountInfo(accountInfo, 0)[0];
  }

  /**
   * Deserializes the {@link MasterEditionV2} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [MasterEditionV2, number] {
    return masterEditionV2Beet.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link MasterEditionV2} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return masterEditionV2Beet.serialize(this);
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link MasterEditionV2} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: MasterEditionV2Args) {
    const instance = MasterEditionV2.fromArgs(args);
    return masterEditionV2Beet.toFixedFromValue(instance).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link MasterEditionV2} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: MasterEditionV2Args,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(MasterEditionV2.byteSize(args), commitment);
  }

  /**
   * Returns a readable version of {@link MasterEditionV2} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: 'Key.' + Key[this.key],
      supply: this.supply,
      maxSupply: this.maxSupply,
    };
  }
}

/**
 * @category Accounts
 * @category generated
 */
export const masterEditionV2Beet = new beet.FixableBeetStruct<MasterEditionV2, MasterEditionV2Args>(
  [
    ['key', keyBeet],
    ['supply', beet.u64],
    ['maxSupply', beet.coption(beet.u64)],
  ],
  MasterEditionV2.fromArgs,
  'MasterEditionV2',
);
