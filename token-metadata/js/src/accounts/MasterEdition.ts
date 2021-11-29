import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import bs58 from 'bs58';
import { Account } from '../../../Account';
import { AnyPublicKey, StringPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { Edition } from './Edition';
import { MetadataKey, MetadataProgram } from '../MetadataProgram';
import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '@metaplex/errors';
import { Buffer } from 'buffer';

type ArgsV1 = {
  key: MetadataKey;
  supply: BN;
  maxSupply?: BN;
  printingMint: StringPublicKey;
  oneTimePrintingAuthorizationMint: StringPublicKey;
};
export class MasterEditionV1Data extends Borsh.Data<ArgsV1> {
  static readonly SCHEMA = this.struct([
    ['key', 'u8'],
    ['supply', 'u64'],
    ['maxSupply', { kind: 'option', type: 'u64' }],
    ['printingMint', 'pubkeyAsString'],
    ['oneTimePrintingAuthorizationMint', 'pubkeyAsString'],
  ]);

  key: MetadataKey;
  supply: BN;
  maxSupply?: BN;
  /// Can be used to mint tokens that give one-time permission to mint a single limited edition.
  printingMint: StringPublicKey;
  /// If you don't know how many printing tokens you are going to need, but you do know
  /// you are going to need some amount in the future, you can use a token from this mint.
  /// Coming back to token metadata with one of these tokens allows you to mint (one time)
  /// any number of printing tokens you want. This is used for instance by Auction Manager
  /// with participation NFTs, where we dont know how many people will bid and need participation
  /// printing tokens to redeem, so we give it ONE of these tokens to use after the auction is over,
  /// because when the auction begins we just dont know how many printing tokens we will need,
  /// but at the end we will. At the end it then burns this token with token-metadata to
  /// get the printing tokens it needs to give to bidders. Each bidder then redeems a printing token
  /// to get their limited editions.
  oneTimePrintingAuthorizationMint: StringPublicKey;

  constructor(args: ArgsV1) {
    super(args);
    this.key = MetadataKey.MasterEditionV1;
  }
}

type ArgsV2 = { key: MetadataKey; supply: BN; maxSupply?: BN };
export class MasterEditionV2Data extends Borsh.Data<ArgsV2> {
  static readonly SCHEMA = this.struct([
    ['key', 'u8'],
    ['supply', 'u64'],
    ['maxSupply', { kind: 'option', type: 'u64' }],
  ]);

  key: MetadataKey;
  supply: BN;
  maxSupply?: BN;

  constructor(args: ArgsV2) {
    super(args);
    this.key = MetadataKey.MasterEditionV2;
  }
}

export type MasterEditionData = MasterEditionV1Data | MasterEditionV2Data;

export class MasterEdition extends Account<MasterEditionData> {
  static readonly EDITION_PREFIX = 'edition';

  constructor(key: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(key, info);

    if (!this.assertOwner(MetadataProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (MasterEdition.isMasterEditionV1(this.info.data)) {
      this.data = MasterEditionV1Data.deserialize(this.info.data);
    } else if (MasterEdition.isMasterEditionV2(this.info.data)) {
      this.data = MasterEditionV2Data.deserialize(this.info.data);
    } else {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }
  }

  static async getPDA(mint: AnyPublicKey) {
    return MetadataProgram.findProgramAddress([
      Buffer.from(MetadataProgram.PREFIX),
      MetadataProgram.PUBKEY.toBuffer(),
      new PublicKey(mint).toBuffer(),
      Buffer.from(MasterEdition.EDITION_PREFIX),
    ]);
  }

  static isCompatible(data: Buffer) {
    return MasterEdition.isMasterEditionV1(data) || MasterEdition.isMasterEditionV2(data);
  }

  static isMasterEditionV1(data: Buffer) {
    return data[0] === MetadataKey.MasterEditionV1;
  }

  static isMasterEditionV2(data: Buffer) {
    return data[0] === MetadataKey.MasterEditionV2;
  }

  async getEditions(connection: Connection) {
    return (
      await MetadataProgram.getProgramAccounts(connection, {
        filters: [
          // Filter for EditionV1 by key
          {
            memcmp: {
              offset: 0,
              bytes: bs58.encode(Buffer.from([MetadataKey.EditionV1])),
            },
          },
          // Filter for assigned to this master edition
          {
            memcmp: {
              offset: 1,
              bytes: this.pubkey.toBase58(),
            },
          },
        ],
      })
    ).map((account) => Edition.from(account));
  }
}
