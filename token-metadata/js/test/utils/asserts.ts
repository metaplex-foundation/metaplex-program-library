import { Test } from 'tape';
import spok, { Specification } from 'spok';
import { COption } from '@metaplex-foundation/beet';
import { PublicKey } from '@solana/web3.js';
import { Specifications } from 'spok';
import { bignum } from '@metaplex-foundation/beet';
import {
  CreateMetadataAccountSetup,
  DataV2,
  Key,
  Metadata,
  TokenStandard,
} from '../../src/mpl-token-metadata';
import BN from 'bn.js';
import { ConfirmedTransactionDetails, MaybeErrorWithCode } from '@metaplex-foundation/amman';

// TODO(thlorenz): move generic asserts into a common spok solana utils library

type Assert = {
  equal<T, U>(actual: T, expected: U, msg?: string): void;
  deepEqual<T, U>(actual: T, expected: U, msg?: string): void;
  ok<T>(value: T, msg?: string): void;
};
export function assertSamePubkey(t: Assert, a: PublicKey | COption<PublicKey>, b: PublicKey) {
  t.equal(a?.toBase58(), b.toBase58(), 'pubkeys are same');
}

export function spokSamePubkey(a: PublicKey | COption<PublicKey>): Specifications<PublicKey> {
  const same = (b: PublicKey) => !!a?.equals(b);

  same.$spec = `spokSamePubkey(${a?.toBase58()})`;
  same.$description = `${a?.toBase58()} equal`;
  return same;
}

export function spokSameBignum(a: bignum): Specification<bignum> {
  const same = (b?: bignum) => b != null && new BN(a).eq(new BN(b));

  same.$spec = `spokSameBignum(${a})`;
  same.$description = `${a} equal`;
  return same;
}

// -----------------
// Cusper specific
// -----------------
// eslint-disable-next-line @typescript-eslint/ban-types
export function assertMatchesError<Err extends Function>(
  t: Test,
  err: MaybeErrorWithCode,
  ty: Err,
  msgRx?: RegExp,
) {
  if (err == null) {
    t.fail(`Expected an error of type ${ty}`);
    return;
  }
  if (err instanceof ty) {
    t.ok(err instanceof ty, ty.name);
  } else {
    t.fail(`Expected error of type ${ty.name} but got ${err.name}`);
  }
  if (msgRx != null) {
    t.match(err.message, msgRx);
  }
}

// eslint-disable-next-line @typescript-eslint/ban-types
export function assertHasError<Err extends Function>(
  t: Test,
  res: ConfirmedTransactionDetails,
  ty: Err,
  msgRx?: RegExp,
) {
  return assertMatchesError(t, res.txSummary.err, ty, msgRx);
}

// -----------------
// Token Metadata Specific
// -----------------
export function assertMetadataAccount(
  t: Test,
  metadataAccount: Metadata,
  setup: CreateMetadataAccountSetup,
  data: DataV2,
  args: { isMutable?: boolean; primarySaleHappened?: boolean } = {},
) {
  const { isMutable = false, primarySaleHappened = false } = args;
  spok(t, metadataAccount, {
    $topic: 'metadataAccount',
    key: Key.MetadataV1,
    updateAuthority: spokSamePubkey(setup.updateAuthority),
    mint: spokSamePubkey(setup.mint),
    data: {
      name: spok.startsWith(data.name),
      symbol: spok.startsWith(data.symbol),
      uri: spok.startsWith(data.uri),
      sellerFeeBasisPoints: data.sellerFeeBasisPoints,
      creators: data.creators,
    },
    isMutable,
    primarySaleHappened,
    editionNonce: spok.number,
    tokenStandard: TokenStandard.FungibleAsset,
    collection: data.collection,
  });
  if (metadataAccount.uses == null || data.uses == null) {
    t.deepEqual(metadataAccount.uses, data.uses, 'uses');
  } else {
    spok(t, metadataAccount.uses, {
      $topic: 'metadataAccount.uses',
      total: spokSameBignum(data.uses.total),
      remaining: spokSameBignum(data.uses.remaining),
      useMethod: data.uses.useMethod,
    });
  }
}
