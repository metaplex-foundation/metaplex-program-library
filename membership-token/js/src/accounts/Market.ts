import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as definedTypes from '../types';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link MarketAccountData}
 */
export type MarketAccountDataArgs = {
  store: web3.PublicKey;
  sellingResource: web3.PublicKey;
  treasuryMint: web3.PublicKey;
  treasuryHolder: web3.PublicKey;
  treasuryOwner: web3.PublicKey;
  owner: web3.PublicKey;
  name: string;
  description: string;
  mutable: boolean;
  price: beet.bignum;
  piecesInOneWallet: beet.COption<beet.bignum>;
  startDate: beet.bignum;
  endDate: beet.COption<beet.bignum>;
  state: definedTypes.MarketState;
};

const marketAccountDiscriminator = [219, 190, 213, 55, 0, 227, 198, 154];
/**
 * Holds the data for the {@link MarketAccount} and provides de/serialization
 * functionality for that data
 */
export class MarketAccountData implements MarketAccountDataArgs {
  private constructor(
    readonly store: web3.PublicKey,
    readonly sellingResource: web3.PublicKey,
    readonly treasuryMint: web3.PublicKey,
    readonly treasuryHolder: web3.PublicKey,
    readonly treasuryOwner: web3.PublicKey,
    readonly owner: web3.PublicKey,
    readonly name: string,
    readonly description: string,
    readonly mutable: boolean,
    readonly price: beet.bignum,
    readonly piecesInOneWallet: beet.COption<beet.bignum>,
    readonly startDate: beet.bignum,
    readonly endDate: beet.COption<beet.bignum>,
    readonly state: definedTypes.MarketState,
  ) {}

  /**
   * Creates a {@link MarketAccountData} instance from the provided args.
   */
  static fromArgs(args: MarketAccountDataArgs) {
    return new MarketAccountData(
      args.store,
      args.sellingResource,
      args.treasuryMint,
      args.treasuryHolder,
      args.treasuryOwner,
      args.owner,
      args.name,
      args.description,
      args.mutable,
      args.price,
      args.piecesInOneWallet,
      args.startDate,
      args.endDate,
      args.state,
    );
  }

  /**
   * Deserializes the {@link MarketAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [MarketAccountData, number] {
    return MarketAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link MarketAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [MarketAccountData, number] {
    return marketAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link MarketAccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: MarketAccountDataArgs) {
    const instance = MarketAccountData.fromArgs(args);
    return marketAccountDataStruct.toFixedFromValue({
      accountDiscriminator: marketAccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link MarketAccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: MarketAccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      MarketAccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Serializes the {@link MarketAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return marketAccountDataStruct.serialize({
      accountDiscriminator: marketAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns a readable version of {@link MarketAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      store: this.store.toBase58(),
      sellingResource: this.sellingResource.toBase58(),
      treasuryMint: this.treasuryMint.toBase58(),
      treasuryHolder: this.treasuryHolder.toBase58(),
      treasuryOwner: this.treasuryOwner.toBase58(),
      owner: this.owner.toBase58(),
      name: this.name,
      description: this.description,
      mutable: this.mutable,
      price: this.price,
      piecesInOneWallet: this.piecesInOneWallet,
      startDate: this.startDate,
      endDate: this.endDate,
      state: this.state,
    };
  }
}

const marketAccountDataStruct = new beet.FixableBeetStruct<
  MarketAccountData,
  MarketAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['store', beetSolana.publicKey],
    ['sellingResource', beetSolana.publicKey],
    ['treasuryMint', beetSolana.publicKey],
    ['treasuryHolder', beetSolana.publicKey],
    ['treasuryOwner', beetSolana.publicKey],
    ['owner', beetSolana.publicKey],
    ['name', beet.utf8String],
    ['description', beet.utf8String],
    ['mutable', beet.bool],
    ['price', beet.u64],
    ['piecesInOneWallet', beet.coption(beet.u64)],
    ['startDate', beet.u64],
    ['endDate', beet.coption(beet.u64)],
    ['state', definedTypes.marketStateEnum],
  ],
  MarketAccountData.fromArgs,
  'MarketAccountData',
);
