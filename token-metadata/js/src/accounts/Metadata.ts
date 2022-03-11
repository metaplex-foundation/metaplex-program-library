import {
  config,
  Borsh,
  Account,
  ERROR_INVALID_ACCOUNT_DATA,
  ERROR_INVALID_OWNER,
  AnyPublicKey,
  StringPublicKey,
  TokenAccount,
} from '@metaplex-foundation/mpl-core';
import { strict as assert } from 'assert';
import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import bs58 from 'bs58';
import { Buffer } from 'buffer';
import { MetadataProgram } from '../MetadataProgram';
import { Edition } from './Edition';
import { MasterEdition } from './MasterEdition';
import { Uses } from './Uses';
import { Collection } from './Collection';
import { MetadataKey, TokenStandard } from './constants';

type CreatorArgs = { address: StringPublicKey; verified: boolean; share: number };
export class Creator extends Borsh.Data<CreatorArgs> {
  static readonly SCHEMA = Creator.struct([
    ['address', 'pubkeyAsString'],
    ['verified', 'u8'],
    ['share', 'u8'],
  ]);

  address: StringPublicKey;
  verified: boolean;
  share: number;
}

type DataArgs = {
  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number;
  creators: Creator[] | null;
};

type DataV2Args = {
  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number;
  creators: Creator[] | null;
  collection: Collection | null;
  uses: Uses | null;
};

export class DataV2 extends Borsh.Data<DataV2Args> {
  static readonly SCHEMA = new Map([
    ...Creator.SCHEMA,
    ...Collection.SCHEMA,
    ...Uses.SCHEMA,
    ...DataV2.struct([
      ['name', 'string'],
      ['symbol', 'string'],
      ['uri', 'string'],
      ['sellerFeeBasisPoints', 'u16'],
      ['creators', { kind: 'option', type: [Creator] }],
      ['collection', { kind: 'option', type: Collection }],
      ['uses', { kind: 'option', type: Uses }],
    ]),
  ]);

  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number;
  creators: Creator[] | null;
  collection: Collection | null;
  uses: Uses | null;
}
export class MetadataDataData extends Borsh.Data<DataArgs> {
  static readonly SCHEMA = new Map([
    ...Creator.SCHEMA,
    ...MetadataDataData.struct([
      ['name', 'string'],
      ['symbol', 'string'],
      ['uri', 'string'],
      ['sellerFeeBasisPoints', 'u16'],
      ['creators', { kind: 'option', type: [Creator] }],
    ]),
  ]);

  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number;
  creators: Creator[] | null;

  constructor(args: DataArgs) {
    super(args);

    const METADATA_REPLACE = new RegExp('\u0000', 'g');
    this.name = args.name.replace(METADATA_REPLACE, '');
    this.uri = args.uri.replace(METADATA_REPLACE, '');
    this.symbol = args.symbol.replace(METADATA_REPLACE, '');
  }
}

type Args = {
  updateAuthority: StringPublicKey;
  mint: StringPublicKey;
  data: MetadataDataData;
  primarySaleHappened: boolean;
  isMutable: boolean;
  editionNonce: number | null;
};
export class MetadataData extends Borsh.Data<Args> {
  static readonly SCHEMA = new Map([
    ...MetadataDataData.SCHEMA,
    ...Collection.SCHEMA,
    ...Uses.SCHEMA,
    ...MetadataData.struct([
      ['key', 'u8'],
      ['updateAuthority', 'pubkeyAsString'],
      ['mint', 'pubkeyAsString'],
      ['data', MetadataDataData],
      ['primarySaleHappened', 'u8'], // bool
      ['isMutable', 'u8'], // bool
      ['editionNonce', { kind: 'option', type: 'u8' }],
      ['tokenStandard', { kind: 'option', type: 'u8' }],
      ['collection', { kind: 'option', type: Collection }],
      ['uses', { kind: 'option', type: Uses }],
    ]),
  ]);

  key: MetadataKey;
  updateAuthority: StringPublicKey;
  mint: StringPublicKey;
  data: MetadataDataData;
  primarySaleHappened: boolean;
  isMutable: boolean;
  editionNonce: number | null;
  tokenStandard: TokenStandard | null;
  collection: Collection | null;
  uses: Uses | null;

  // set lazy
  masterEdition?: StringPublicKey;
  edition?: StringPublicKey;

  constructor(args: Args) {
    super(args);
    this.key = MetadataKey.MetadataV1;
  }
}

export class Metadata extends Account<MetadataData> {
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);

    if (!this.assertOwner(MetadataProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    assert(this.info != null, 'account info needs to be defined');
    if (!Metadata.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = MetadataData.deserialize(this.info.data);
  }

  static isCompatible(data: Buffer) {
    return data[0] === MetadataKey.MetadataV1;
  }

  static async getPDA(mint: AnyPublicKey) {
    return MetadataProgram.findProgramAddress([
      Buffer.from(MetadataProgram.PREFIX),
      MetadataProgram.PUBKEY.toBuffer(),
      new PublicKey(mint).toBuffer(),
    ]);
  }

  static async findMany(
    connection: Connection,
    filters: {
      mint?: AnyPublicKey;
      updateAuthority?: AnyPublicKey;
      creators?: AnyPublicKey[];
    } = {},
  ) {
    const baseFilters = [
      // Filter for MetadataV1 by key
      {
        memcmp: {
          offset: 0,
          bytes: bs58.encode(Buffer.from([MetadataKey.MetadataV1])),
        },
      },
      // Filter for assigned to update authority
      filters.updateAuthority && {
        memcmp: {
          offset: 1,
          bytes: new PublicKey(filters.updateAuthority).toBase58(),
        },
      },
      // Filter for assigned to mint
      filters.mint && {
        memcmp: {
          offset: 33,
          bytes: new PublicKey(filters.mint).toBase58(),
        },
      },
    ].filter(Boolean);

    if (filters.creators) {
      return (
        await Promise.all(
          Array.from(Array(config.maxCreatorLimit).keys()).reduce(
            (prev, i) => [
              ...prev,
              ...filters.creators.map((pubkey) =>
                MetadataProgram.getProgramAccounts(connection, {
                  filters: [
                    ...baseFilters,
                    {
                      memcmp: {
                        offset: computeCreatorOffset(i),
                        bytes: new PublicKey(pubkey).toBase58(),
                      },
                    },
                  ],
                }),
              ),
            ],
            [],
          ),
        )
      )
        .flat()
        .map((account) => Metadata.from(account));
    } else {
      return (await MetadataProgram.getProgramAccounts(connection, { filters: baseFilters })).map(
        (account) => Metadata.from(account),
      );
    }
  }

  static async findByMint(connection: Connection, mint: AnyPublicKey): Promise<Metadata> {
    const pda = await Metadata.getPDA(mint);

    return Metadata.load(connection, pda);
  }

  static async findByOwnerV2(connection: Connection, owner: AnyPublicKey) {
    const accounts = await TokenAccount.getTokenAccountsByOwner(connection, owner);
    const accountsWithAmount = accounts
      .map(({ data }) => data)
      .filter(({ amount }) => amount?.toNumber() > 0);

    return (
      await Promise.all(
        accountsWithAmount.map(({ mint }) => Metadata.findMany(connection, { mint })),
      )
    ).flat();
  }

  static async findByOwnerV3(connection: Connection, owner: AnyPublicKey): Promise<Metadata[]> {
    const tokenInfo = await Metadata.findInfoByOwner(connection, owner);

    return Array.from(tokenInfo.entries()).map(([pubkey, info]) => new Metadata(pubkey, info));
  }

  static async findInfoByOwner(
    connection: Connection,
    owner: AnyPublicKey,
  ): Promise<Map<AnyPublicKey, AccountInfo<Buffer>>> {
    const accounts = await TokenAccount.getTokenAccountsByOwner(connection, owner);

    const metadataPdaLookups = accounts.reduce((memo, { data }) => {
      // Only include tokens where amount equal to 1.
      // Note: This is not the same as mint supply.
      // NFTs by definition have supply of 1, but an account balance > 1 implies a mint supply > 1.
      return data.amount?.eq(new BN(1)) ? [...memo, Metadata.getPDA(data.mint)] : memo;
    }, []);

    const metadataAddresses = await Promise.all(metadataPdaLookups);

    return Account.getInfos(connection, metadataAddresses);
  }

  static async findDataByOwner(
    connection: Connection,
    owner: AnyPublicKey,
  ): Promise<MetadataData[]> {
    const tokenInfo = await Metadata.findInfoByOwner(connection, owner);

    return Array.from(tokenInfo.values()).map((m) => MetadataData.deserialize(m.data));
  }

  static async getEdition(connection: Connection, mint: AnyPublicKey) {
    const pda = await Edition.getPDA(mint);
    const info = await Account.getInfo(connection, pda);
    const key = info?.data[0];

    switch (key) {
      case MetadataKey.EditionV1:
        return new Edition(pda, info);
      case MetadataKey.MasterEditionV1:
      case MetadataKey.MasterEditionV2:
        return new MasterEdition(pda, info);
      default:
        return;
    }
  }
}

export const MAX_NAME_LENGTH = 32;
export const MAX_SYMBOL_LENGTH = 10;
export const MAX_URI_LENGTH = 200;
export const MAX_CREATOR_LEN = 32 + 1 + 1;

export const computeCreatorOffset = (index: number) => {
  return (
    1 + // key
    32 + // update auth
    32 + // mint
    4 + // name string length
    MAX_NAME_LENGTH + // name
    4 + // uri string length
    MAX_URI_LENGTH + // uri
    4 + // symbol string length
    MAX_SYMBOL_LENGTH + // symbol
    2 + // seller fee basis points
    1 + // whether or not there is a creators vec
    4 + // creators vec length
    index * MAX_CREATOR_LEN
  );
};
