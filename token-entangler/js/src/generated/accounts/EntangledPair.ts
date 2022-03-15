import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';

/**
 * Arguments used to create {@link EntangledPairAccountData}
 */
export type EntangledPairAccountDataArgs = {
  treasuryMint: web3.PublicKey;
  mintA: web3.PublicKey;
  mintB: web3.PublicKey;
  tokenAEscrow: web3.PublicKey;
  tokenBEscrow: web3.PublicKey;
  authority: web3.PublicKey;
  bump: number;
  tokenAEscrowBump: number;
  tokenBEscrowBump: number;
  price: beet.bignum;
  paid: boolean;
  paysEveryTime: boolean;
};

const entangledPairAccountDiscriminator = [133, 118, 20, 210, 1, 54, 172, 116];
/**
 * Holds the data for the {@link EntangledPairAccount} and provides de/serialization
 * functionality for that data
 */
export class EntangledPairAccountData implements EntangledPairAccountDataArgs {
  private constructor(
    readonly treasuryMint: web3.PublicKey,
    readonly mintA: web3.PublicKey,
    readonly mintB: web3.PublicKey,
    readonly tokenAEscrow: web3.PublicKey,
    readonly tokenBEscrow: web3.PublicKey,
    readonly authority: web3.PublicKey,
    readonly bump: number,
    readonly tokenAEscrowBump: number,
    readonly tokenBEscrowBump: number,
    readonly price: beet.bignum,
    readonly paid: boolean,
    readonly paysEveryTime: boolean,
  ) {}

  /**
   * Creates a {@link EntangledPairAccountData} instance from the provided args.
   */
  static fromArgs(args: EntangledPairAccountDataArgs) {
    return new EntangledPairAccountData(
      args.treasuryMint,
      args.mintA,
      args.mintB,
      args.tokenAEscrow,
      args.tokenBEscrow,
      args.authority,
      args.bump,
      args.tokenAEscrowBump,
      args.tokenBEscrowBump,
      args.price,
      args.paid,
      args.paysEveryTime,
    );
  }

  /**
   * Deserializes the {@link EntangledPairAccountData} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0,
  ): [EntangledPairAccountData, number] {
    return EntangledPairAccountData.deserialize(accountInfo.data, offset);
  }

  /**
   * Deserializes the {@link EntangledPairAccountData} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [EntangledPairAccountData, number] {
    return entangledPairAccountDataStruct.deserialize(buf, offset);
  }

  /**
   * Serializes the {@link EntangledPairAccountData} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return entangledPairAccountDataStruct.serialize({
      accountDiscriminator: entangledPairAccountDiscriminator,
      ...this,
    });
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link EntangledPairAccountData}
   */
  static get byteSize() {
    return entangledPairAccountDataStruct.byteSize;
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link EntangledPairAccountData} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment,
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      EntangledPairAccountData.byteSize,
      commitment,
    );
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link EntangledPairAccountData} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === EntangledPairAccountData.byteSize;
  }

  /**
   * Returns a readable version of {@link EntangledPairAccountData} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      treasuryMint: this.treasuryMint.toBase58(),
      mintA: this.mintA.toBase58(),
      mintB: this.mintB.toBase58(),
      tokenAEscrow: this.tokenAEscrow.toBase58(),
      tokenBEscrow: this.tokenBEscrow.toBase58(),
      authority: this.authority.toBase58(),
      bump: this.bump,
      tokenAEscrowBump: this.tokenAEscrowBump,
      tokenBEscrowBump: this.tokenBEscrowBump,
      price: this.price,
      paid: this.paid,
      paysEveryTime: this.paysEveryTime,
    };
  }
}

const entangledPairAccountDataStruct = new beet.BeetStruct<
  EntangledPairAccountData,
  EntangledPairAccountDataArgs & {
    accountDiscriminator: number[];
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['treasuryMint', beetSolana.publicKey],
    ['mintA', beetSolana.publicKey],
    ['mintB', beetSolana.publicKey],
    ['tokenAEscrow', beetSolana.publicKey],
    ['tokenBEscrow', beetSolana.publicKey],
    ['authority', beetSolana.publicKey],
    ['bump', beet.u8],
    ['tokenAEscrowBump', beet.u8],
    ['tokenBEscrowBump', beet.u8],
    ['price', beet.u64],
    ['paid', beet.bool],
    ['paysEveryTime', beet.bool],
  ],
  EntangledPairAccountData.fromArgs,
  'EntangledPairAccountData',
);
