import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link MetadataAccountData}
 */
export type MetadataAccountDataArgs = {
  key: definedTypes.Key;
  updateAuthority: web3.PublicKey;
  mint: web3.PublicKey;
  data: definedTypes.Data;
  primarySaleHappened: boolean;
  isMutable: boolean;
  editionNonce: beet.COption<number>;
  tokenStandard: beet.COption<definedTypes.TokenStandard>;
  collection: beet.COption<definedTypes.Collection>;
  uses: beet.COption<definedTypes.Uses>;
};

const metadataAccountDiscriminator = [72, 11, 121, 26, 111, 181, 85, 93];
/**
 * Holds the data for the {@link MetadataAccount} and provides de/serialization
 * functionality for that data
 */
export class MetadataAccountData implements MetadataAccountDataArgs {
  private constructor(
    readonly key: definedTypes.Key,
    readonly updateAuthority: web3.PublicKey,
    readonly mint: web3.PublicKey,
    readonly data: definedTypes.Data,
    readonly primarySaleHappened: boolean,
    readonly isMutable: boolean,
    readonly editionNonce: beet.COption<number>,
    readonly tokenStandard: beet.COption<definedTypes.TokenStandard>,
    readonly collection: beet.COption<definedTypes.Collection>,
    readonly uses: beet.COption<definedTypes.Uses>,
  ) {}

  /**
   * Creates a {@link MetadataAccountData} instance from the provided args.
   */
  static fromArgs(args: MetadataAccountDataArgs) {
    return new MetadataAccountData(
      args.key,
      args.updateAuthority,
      args.mint,
      args.data,
      args.primarySaleHappened,
      args.isMutable,
      args.editionNonce,
      args.tokenStandard,
      args.collection,
      args.uses,
    );
  }

  /**
   * Deserializes the {@link MetadataAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [MetadataAccountData, number] {
    return MetadataAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link MetadataAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [MetadataAccountData, number] {
    return metadataAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link MetadataAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return metadataAccountDataStruct.serialize({
      accountDiscriminator: metadataAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link MetadataAccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: MetadataAccountDataArgs) {
    const instance = MetadataAccountData.fromArgs(args);
    return metadataAccountDataStruct.toFixedFromValue({
      accountDiscriminator: metadataAccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link MetadataAccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: MetadataAccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      MetadataAccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Returns a readable version of {@link MetadataAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: this.key,
      updateAuthority: this.updateAuthority.toBase58(),
      mint: this.mint.toBase58(),
      data: this.data,
      primarySaleHappened: this.primarySaleHappened,
      isMutable: this.isMutable,
      editionNonce: this.editionNonce,
      tokenStandard: this.tokenStandard,
      collection: this.collection,
      uses: this.uses,
    };
  }
}

const metadataAccountDataStruct = new beet.FixableBeetStruct<
  MetadataAccountData,
  MetadataAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['key', definedTypes.keyStruct],
    ['updateAuthority', beetSolana.publicKey],
    ['mint', beetSolana.publicKey],
    ['data', definedTypes.dataStruct],
    ['primarySaleHappened', beet.bool],
    ['isMutable', beet.bool],
    ['editionNonce', beet.coption(beet.u8)],
    ['tokenStandard', beet.coption(definedTypes.tokenStandardEnum)],
    ['collection', beet.coption(definedTypes.collectionStruct)],
    ['uses', beet.coption(definedTypes.usesStruct)],
  ],
  MetadataAccountData.fromArgs,
  'MetadataAccountData',
);
