import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as definedTypes from '../types';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link SellingResourceAccountData}
 */
export type SellingResourceAccountDataArgs = {
  store: web3.PublicKey;
  owner: web3.PublicKey;
  resource: web3.PublicKey;
  vault: web3.PublicKey;
  vaultOwner: web3.PublicKey;
  supply: beet.bignum;
  maxSupply: beet.COption<beet.bignum>;
  state: definedTypes.SellingResourceState;
};

const sellingResourceAccountDiscriminator = [15, 32, 69, 235, 249, 39, 18, 167];
/**
 * Holds the data for the {@link SellingResourceAccount} and provides de/serialization
 * functionality for that data
 */
export class SellingResourceAccountData implements SellingResourceAccountDataArgs {
  private constructor(
    readonly store: web3.PublicKey,
    readonly owner: web3.PublicKey,
    readonly resource: web3.PublicKey,
    readonly vault: web3.PublicKey,
    readonly vaultOwner: web3.PublicKey,
    readonly supply: beet.bignum,
    readonly maxSupply: beet.COption<beet.bignum>,
    readonly state: definedTypes.SellingResourceState,
  ) {}

  /**
   * Creates a {@link SellingResourceAccountData} instance from the provided args.
   */
  static fromArgs(args: SellingResourceAccountDataArgs) {
    return new SellingResourceAccountData(
      args.store,
      args.owner,
      args.resource,
      args.vault,
      args.vaultOwner,
      args.supply,
      args.maxSupply,
      args.state,
    );
  }

  /**
   * Deserializes the {@link SellingResourceAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [SellingResourceAccountData, number] {
    return SellingResourceAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link SellingResourceAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [SellingResourceAccountData, number] {
    return sellingResourceAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link SellingResourceAccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: SellingResourceAccountDataArgs) {
    const instance = SellingResourceAccountData.fromArgs(args);
    return sellingResourceAccountDataStruct.toFixedFromValue({
      accountDiscriminator: sellingResourceAccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link SellingResourceAccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: SellingResourceAccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      SellingResourceAccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Serializes the {@link SellingResourceAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return sellingResourceAccountDataStruct.serialize({
      accountDiscriminator: sellingResourceAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns a readable version of {@link SellingResourceAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      store: this.store.toBase58(),
      owner: this.owner.toBase58(),
      resource: this.resource.toBase58(),
      vault: this.vault.toBase58(),
      vaultOwner: this.vaultOwner.toBase58(),
      supply: this.supply,
      maxSupply: this.maxSupply,
      state: this.state,
    };
  }
}

const sellingResourceAccountDataStruct = new beet.FixableBeetStruct<
  SellingResourceAccountData,
  SellingResourceAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['store', beetSolana.publicKey],
    ['owner', beetSolana.publicKey],
    ['resource', beetSolana.publicKey],
    ['vault', beetSolana.publicKey],
    ['vaultOwner', beetSolana.publicKey],
    ['supply', beet.u64],
    ['maxSupply', beet.coption(beet.u64)],
    ['state', definedTypes.sellingResourceStateEnum],
  ],
  SellingResourceAccountData.fromArgs,
  'SellingResourceAccountData',
);
