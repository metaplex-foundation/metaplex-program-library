import * as definedTypes from '../types';
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link ReservationListV2AccountData}
 */
export type ReservationListV2AccountDataArgs = {
  key: definedTypes.Key;
  masterEdition: web3.PublicKey;
  supplySnapshot: beet.COption<beet.bignum>;
  reservations: definedTypes.Reservation[];
  totalReservationSpots: beet.bignum;
  currentReservationSpots: beet.bignum;
};

const reservationListV2AccountDiscriminator = [193, 233, 97, 55, 245, 135, 103, 218];
/**
 * Holds the data for the {@link ReservationListV2Account} and provides de/serialization
 * functionality for that data
 */
export class ReservationListV2AccountData implements ReservationListV2AccountDataArgs {
  private constructor(
    readonly key: definedTypes.Key,
    readonly masterEdition: web3.PublicKey,
    readonly supplySnapshot: beet.COption<beet.bignum>,
    readonly reservations: definedTypes.Reservation[],
    readonly totalReservationSpots: beet.bignum,
    readonly currentReservationSpots: beet.bignum,
  ) {}

  /**
   * Creates a {@link ReservationListV2AccountData} instance from the provided args.
   */
  static fromArgs(args: ReservationListV2AccountDataArgs) {
    return new ReservationListV2AccountData(
      args.key,
      args.masterEdition,
      args.supplySnapshot,
      args.reservations,
      args.totalReservationSpots,
      args.currentReservationSpots,
    );
  }

  /**
   * Deserializes the {@link ReservationListV2AccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [ReservationListV2AccountData, number] {
    return ReservationListV2AccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link ReservationListV2AccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [ReservationListV2AccountData, number] {
    return reservationListV2AccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link ReservationListV2AccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return reservationListV2AccountDataStruct.serialize({
      accountDiscriminator: reservationListV2AccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link ReservationListV2AccountData} for the provided args.
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   */
  static byteSize(args: ReservationListV2AccountDataArgs) {
    const instance = ReservationListV2AccountData.fromArgs(args);
    return reservationListV2AccountDataStruct.toFixedFromValue({
      accountDiscriminator: reservationListV2AccountDiscriminator,
      ...instance,
    }).byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link ReservationListV2AccountData} data from rent
   *
   * @param args need to be provided since the byte size for this account
   * depends on them
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    args: ReservationListV2AccountDataArgs,
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      ReservationListV2AccountData.byteSize(args),
      commitment,
    );
  }

  /**
   * Returns a readable version of {@link ReservationListV2AccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      key: this.key,
      masterEdition: this.masterEdition.toBase58(),
      supplySnapshot: this.supplySnapshot,
      reservations: this.reservations,
      totalReservationSpots: this.totalReservationSpots,
      currentReservationSpots: this.currentReservationSpots,
    };
  }
}

const reservationListV2AccountDataStruct = new beet.FixableBeetStruct<
  ReservationListV2AccountData,
  ReservationListV2AccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['key', definedTypes.keyStruct],
    ['masterEdition', beetSolana.publicKey],
    ['supplySnapshot', beet.coption(beet.u64)],
    ['reservations', beet.array(definedTypes.reservationStruct)],
    ['totalReservationSpots', beet.u64],
    ['currentReservationSpots', beet.u64],
  ],
  ReservationListV2AccountData.fromArgs,
  'ReservationListV2AccountData',
);
