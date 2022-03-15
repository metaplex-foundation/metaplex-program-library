import { Borsh } from '@metaplex-foundation/mpl-core';
import { MetadataKey, UseMethod } from '.';

type UsesArgs = { useMethod: UseMethod; total: number; remaining: number };
export class Uses extends Borsh.Data<UsesArgs> {
  static readonly SCHEMA = Uses.struct([
    ['useMethod', 'u8'],
    ['total', 'u64'],
    ['remaining', 'u64'],
  ]);
  useMethod: UseMethod;
  total: number;
  remaining: number;

  constructor(args: UsesArgs) {
    super(args);
    this.useMethod = args.useMethod;
    this.total = args.total;
    this.remaining = args.remaining;
  }
}

type UseAuthorityRecordArgs = { allowedUses: number; bump: number };
export class UseAuthorityRecord extends Borsh.Data<UseAuthorityRecordArgs> {
  static readonly SCHEMA = UseAuthorityRecord.struct([
    ['key', 'u8'],
    ['allowedUses', 'u64'],
    ['bump', 'u8'],
  ]);
  key: MetadataKey;
  allowedUses: number;
  bump: number;

  constructor(args: UseAuthorityRecordArgs) {
    super(args);
    this.key = MetadataKey.UseAuthorityRecord;
    this.allowedUses = args.allowedUses;
    this.bump = args.bump;
  }
}
