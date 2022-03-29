import { Borsh, StringPublicKey } from '@metaplex-foundation/mpl-core';
import { MetadataKey } from './constants';

type Args = { key: StringPublicKey; verified: boolean };
export class Collection extends Borsh.Data<Args> {
  static readonly SCHEMA = Collection.struct([
    ['verified', 'u8'],
    ['key', 'pubkeyAsString'],
  ]);
  key: StringPublicKey;
  verified: boolean;

  constructor(args: Args) {
    super(args);
    this.key = args.key;
    this.verified = args.verified;
  }
}

type CollectionAuthorityRecordArgs = { bump: number };
export class CollctionAuthorityRecord extends Borsh.Data<CollectionAuthorityRecordArgs> {
  static readonly SCHEMA = CollctionAuthorityRecord.struct([
    ['key', 'u8'],
    ['bump', 'u8'],
  ]);
  key: MetadataKey;
  bump: number;

  constructor(args: CollectionAuthorityRecordArgs) {
    super(args);
    this.key = MetadataKey.CollectionAuthorityRecord;
    this.bump = args.bump;
  }
}
