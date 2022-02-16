import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as definedTypes from '../types';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link CandyMachineAccountData}
 */
export type CandyMachineAccountDataArgs = {
  authority: web3.PublicKey;
  wallet: web3.PublicKey;
  tokenMint: beet.COption<web3.PublicKey>;
  itemsRedeemed: beet.bignum;
  data: definedTypes.CandyMachineData;
};

const candyMachineAccountDiscriminator = [51, 173, 177, 113, 25, 241, 109, 189];
/**
 * Holds the data for the {@link CandyMachineAccount} and provides de/serialization
 * functionality for that data
 */
export class CandyMachineAccountData implements CandyMachineAccountDataArgs {
  private constructor(
    readonly authority: web3.PublicKey,
    readonly wallet: web3.PublicKey,
    readonly tokenMint: beet.COption<web3.PublicKey>,
    readonly itemsRedeemed: beet.bignum,
    readonly data: definedTypes.CandyMachineData,
  ) {}

  /**
   * Creates a {@link CandyMachineAccountData} instance from the provided args.
   */
  static fromArgs(args: CandyMachineAccountDataArgs) {
    return new CandyMachineAccountData(
      args.authority,
      args.wallet,
      args.tokenMint,
      args.itemsRedeemed,
      args.data,
    );
  }

  /**
   * Deserializes the {@link CandyMachineAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [CandyMachineAccountData, number] {
    return CandyMachineAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link CandyMachineAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [CandyMachineAccountData, number] {
    return candyMachineAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link CandyMachineAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return candyMachineAccountDataStruct.serialize({
      accountDiscriminator: candyMachineAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link CandyMachineAccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: CandyMachineAccountDataArgs) {
    const instance = CandyMachineAccountData.fromArgs(args);
    return candyMachineAccountDataStruct.toFixedFromValue({
      accountDiscriminator: candyMachineAccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link CandyMachineAccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: CandyMachineAccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      CandyMachineAccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Returns a readable version of {@link CandyMachineAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      authority: this.authority.toBase58(),
      wallet: this.wallet.toBase58(),
      tokenMint: this.tokenMint,
      itemsRedeemed: this.itemsRedeemed,
      data: this.data,
    };
  }
}

const candyMachineAccountDataStruct = new beet.FixableBeetStruct<
  CandyMachineAccountData,
  CandyMachineAccountDataArgs & {
    accountDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['authority', beetSolana.publicKey],
    ['wallet', beetSolana.publicKey],
    ['tokenMint', beet.coption(beetSolana.publicKey)],
    ['itemsRedeemed', beet.u64],
    ['data', definedTypes.candyMachineDataStruct],
  ],
  CandyMachineAccountData.fromArgs,
  'CandyMachineAccountData',
);
