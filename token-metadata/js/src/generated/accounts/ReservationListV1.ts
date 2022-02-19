import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link ReservationListV1AccountData}
 */
export type ReservationListV1AccountDataArgs = {
  key: definedTypes.Key;
  masterEdition: web3.PublicKey;
  supplySnapshot: beet.COption<beet.bignum>;
  reservations: definedTypes.ReservationV1[];
};

const reservationListV1AccountDiscriminator = [239, 79, 12, 206, 116, 153, 1, 140];
/**
 * Holds the data for the {@link ReservationListV1Account} and provides de/serialization
 * functionality for that data
 */
export class ReservationListV1AccountData implements ReservationListV1AccountDataArgs {
  private constructor(
    readonly key: definedTypes.Key,
    readonly masterEdition: web3.PublicKey,
    readonly supplySnapshot: beet.COption<beet.bignum>,
    readonly reservations: definedTypes.ReservationV1[],
  ) {}

  /**
   * Creates a {@link ReservationListV1AccountData} instance from the provided args.
   */
  static fromArgs(args: ReservationListV1AccountDataArgs) {
    return new ReservationListV1AccountData(
      args.key,
      args.masterEdition,
      args.supplySnapshot,
      args.reservations,
    );
  }

  /**
   * Deserializes the {@link ReservationListV1AccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [ReservationListV1AccountData, number] {
    return ReservationListV1AccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link ReservationListV1AccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [ReservationListV1AccountData, number] {
    return reservationListV1AccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link ReservationListV1AccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return reservationListV1AccountDataStruct.serialize({
      accountDiscriminator: reservationListV1AccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link ReservationListV1AccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: ReservationListV1AccountDataArgs) {
    const instance = ReservationListV1AccountData.fromArgs(args);
    return reservationListV1AccountDataStruct.toFixedFromValue({
      accountDiscriminator: reservationListV1AccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link ReservationListV1AccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: ReservationListV1AccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      ReservationListV1AccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Returns a readable version of {@link ReservationListV1AccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: this.key,
      masterEdition: this.masterEdition.toBase58(),
      supplySnapshot: this.supplySnapshot,
      reservations: this.reservations,
    };
  }
}

const reservationListV1AccountDataStruct = new beet.FixableBeetStruct<
  ReservationListV1AccountData,
  ReservationListV1AccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['key', definedTypes.keyStruct],
    ['masterEdition', beetSolana.publicKey],
    ['supplySnapshot', beet.coption(beet.u64)],
    ['reservations', beet.array(definedTypes.reservationV1Struct)],
  ],
  ReservationListV1AccountData.fromArgs,
  'ReservationListV1AccountData',
);
