import { Test } from 'tape';
import spok from 'spok';
import { COption } from '@metaplex-foundation/beet';
import { PublicKey } from '@solana/web3.js';
import { Specifications } from 'spok';
import {
  CreateMetadataAccountSetup,
  DataV2,
  Key,
  Metadata,
  TokenStandard,
} from '../../src/mpl-token-metadata';

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

// -----------------
// Token Metadata Specific
// -----------------
export function assertMetadataAccount(
  t: Test,
  metadataAccount: Metadata,
  setup: CreateMetadataAccountSetup,
  data: DataV2,
  args: { isMutable: boolean; primarySaleHappened: boolean },
) {
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
    isMutable: args.isMutable,
    primarySaleHappened: args.primarySaleHappened,
    editionNonce: spok.number,
    tokenStandard: TokenStandard.FungibleAsset,
    collection: data.collection,
    uses: data.uses,
  });
}
