import { bignum } from '@metaplex-foundation/beet';
import BN from 'bn.js';
import { Specification } from 'spok';
import { Test } from 'tape';

export * from './errors';

export function spokSameBignum(a?: BN | bignum): Specification<bignum> {
  const same = (b?: BN | bignum) => {
    if (a == null && b == null) {
      return true;
    }
    if (a == null) {
      return false;
    }

    return b != null && new BN(a).eq(new BN(b));
  };

  same.$spec = `spokSameBignum(${a})`;
  same.$description = `${a} equal`;
  return same;
}

export function spokSameBigint(a?: BN | bigint): Specification<bigint> {
  const same = (b?: BN | bigint) => {
    if (a == null && b == null) {
      return true;
    }
    if (a == null) {
      return false;
    }

    return b != null && new BN(a.toString()).eq(new BN(b.toString()));
  };

  same.$spec = `spokSameBigint(${a})`;
  same.$description = `${a} equal`;
  return same;
}

export function assertIsNotNull<T>(t: Test, x: T | null | undefined): asserts x is T {
  t.ok(x, 'should be non null');
}
