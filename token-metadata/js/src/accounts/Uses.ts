import { Borsh } from '@metaplex-foundation/mpl-core';
import { UseMethod } from '.';

type Args = { useMethod: UseMethod; total: number; remaining: number };
export class Uses extends Borsh.Data<Args> {
  static readonly SCHEMA = Uses.struct([
    ['useMethod', 'u8'],
    ['total', 'u64'],
    ['remaining', 'u64'],
  ]);
  useMethod: UseMethod;
  total: number;
  remaining: number;

  constructor(args: Args) {
    super(args);
    this.useMethod = args.useMethod;
    this.total = args.total;
    this.remaining = args.remaining;
  }
}
