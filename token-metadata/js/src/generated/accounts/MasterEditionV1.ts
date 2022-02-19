import * as definedTypes from '../types';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link MasterEditionV1AccountData}
 */
export type MasterEditionV1AccountDataArgs = {
  key: definedTypes.Key;
  supply: beet.bignum;
  maxSupply: beet.COption<beet.bignum>;
  printingMint: web3.PublicKey;
  oneTimePrintingAuthorizationMint: web3.PublicKey;
};

const masterEditionV1AccountDiscriminator = [79, 165, 41, 167, 180, 191, 141, 185];
/**
 * Holds the data for the {@link MasterEditionV1Account} and provides de/serialization
 * functionality for that data
 */
export class MasterEditionV1AccountData implements MasterEditionV1AccountDataArgs {
  private constructor(
    readonly key: definedTypes.Key,
    readonly supply: beet.bignum,
    readonly maxSupply: beet.COption<beet.bignum>,
    readonly printingMint: web3.PublicKey,
    readonly oneTimePrintingAuthorizationMint: web3.PublicKey,
  ) {}

  /**
   * Creates a {@link MasterEditionV1AccountData} instance from the provided args.
   */
  static fromArgs(args: MasterEditionV1AccountDataArgs) {
    return new MasterEditionV1AccountData(
      args.key,
      args.supply,
      args.maxSupply,
      args.printingMint,
      args.oneTimePrintingAuthorizationMint,
    );
  }

  /**
   * Deserializes the {@link MasterEditionV1AccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [MasterEditionV1AccountData, number] {
    return MasterEditionV1AccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link MasterEditionV1AccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [MasterEditionV1AccountData, number] {
    return masterEditionV1AccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link MasterEditionV1AccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return masterEditionV1AccountDataStruct.serialize({
      accountDiscriminator: masterEditionV1AccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link MasterEditionV1AccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: MasterEditionV1AccountDataArgs) {
    const instance = MasterEditionV1AccountData.fromArgs(args);
    return masterEditionV1AccountDataStruct.toFixedFromValue({
      accountDiscriminator: masterEditionV1AccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link MasterEditionV1AccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: MasterEditionV1AccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      MasterEditionV1AccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Returns a readable version of {@link MasterEditionV1AccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: this.key,
      supply: this.supply,
      maxSupply: this.maxSupply,
      printingMint: this.printingMint.toBase58(),
      oneTimePrintingAuthorizationMint: this.oneTimePrintingAuthorizationMint.toBase58(),
    };
  }
}

const masterEditionV1AccountDataStruct = new beet.FixableBeetStruct<
  MasterEditionV1AccountData,
  MasterEditionV1AccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['key', definedTypes.keyStruct],
    ['supply', beet.u64],
    ['maxSupply', beet.coption(beet.u64)],
    ['printingMint', beetSolana.publicKey],
    ['oneTimePrintingAuthorizationMint', beetSolana.publicKey],
  ],
  MasterEditionV1AccountData.fromArgs,
  'MasterEditionV1AccountData',
);
