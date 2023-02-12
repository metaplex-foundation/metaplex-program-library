import { bignum, COption } from '@metaplex-foundation/beet';
import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { Assert, Specification, Specifications } from 'spok';
import { Test } from 'tape';

export * from './errors';

export const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
);

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

export function spokSamePubkey(a: PublicKey | COption<PublicKey>): Specifications<PublicKey> {
  const same = (b: PublicKey | null | undefined) => b != null && !!a?.equals(b);

  same.$spec = `spokSamePubkey(${a?.toBase58()})`;
  same.$description = `${a?.toBase58()} equal`;
  return same;
}

export function assertIsNotNull<T>(t: Test, x: T | null | undefined): asserts x is T {
  t.ok(x, 'should be non null');
}

export function assertSamePubkey(t: Assert, a: PublicKey | COption<PublicKey>, b: PublicKey) {
  t.equal(a?.toBase58(), b.toBase58(), 'pubkeys are same');
}
