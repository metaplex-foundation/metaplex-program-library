import { Borsh } from '@metaplex/utils';
import { AnyPublicKey, StringPublicKey } from '@metaplex/types';
import { MetadataProgram, MetadataKey } from '../MetadataProgram';
import { AccountInfo, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { Account } from '../../../Account';
import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '@metaplex/errors';
import { Buffer } from 'buffer';

type Args = { key: MetadataKey; parent: StringPublicKey; edition: BN };
export class EditionData extends Borsh.Data<Args> {
  static readonly SCHEMA = this.struct([
    ['key', 'u8'],
    ['parent', 'pubkeyAsString'],
    ['edition', 'u64'],
  ]);
  key: MetadataKey;
  /// Points at MasterEdition struct
  parent: StringPublicKey;
  /// Starting at 0 for master record, this is incremented for each edition minted.
  edition: BN;

  constructor(args: Args) {
    super(args);
    this.key = MetadataKey.EditionV1;
  }
}

export class Edition extends Account<EditionData> {
  static readonly EDITION_PREFIX = 'edition';

  constructor(key: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(key, info);

    if (!this.assertOwner(MetadataProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (!Edition.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = EditionData.deserialize(this.info.data);
  }

  static async getPDA(mint: AnyPublicKey) {
    return MetadataProgram.findProgramAddress([
      Buffer.from(MetadataProgram.PREFIX),
      MetadataProgram.PUBKEY.toBuffer(),
      new PublicKey(mint).toBuffer(),
      Buffer.from(Edition.EDITION_PREFIX),
    ]);
  }

  static isCompatible(data: Buffer) {
    return data[0] === MetadataKey.EditionV1;
  }
}
