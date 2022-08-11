import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';
import { Metadata } from '../generated/accounts/Metadata';
import { collectionBeet } from '../generated/types/Collection';
import { collectionDetailsBeet } from '../generated/types/CollectionDetails';
import { dataBeet } from '../generated/types/Data';
import { keyBeet } from '../generated/types/Key';
import { tokenStandardBeet } from '../generated/types/TokenStandard';
import { usesBeet } from '../generated/types/Uses';

const NONE_BYTE_SIZE = beet.coptionNone('').byteSize;

/**
 * This is a custom deserializer for TokenMetadata in order to mitigate acounts with corrupted
 * data on chain.
 *
 * Instead of failing the deserialization for the section that is possibly corrupt it just returns
 * `null` for the fields that would normally be stored in that section.
 *
 * This deserializer matches the [fix implemented in the Rust program](https://github.com/metaplex-foundation/metaplex-program-library/blob/df36da5a78fb17e1690247b8041b761d27c83b1b/token-metadata/program/src/deser.rs#L6).
 * Also @see ../../../program/src/deser.rs
 */
export function deserialize(buf: Buffer, offset = 0): [Metadata, number] {
  let cursor = offset;

  // key
  const key = keyBeet.read(buf, cursor);
  cursor += keyBeet.byteSize;

  // updateAuthority
  const updateAuthority = beetSolana.publicKey.read(buf, cursor);
  cursor += beetSolana.publicKey.byteSize;

  // mint
  const mint = beetSolana.publicKey.read(buf, cursor);
  cursor += beetSolana.publicKey.byteSize;

  // data
  const [data, dataDelta] = dataBeet.deserialize(buf, cursor);
  cursor = dataDelta;

  // primarySaleHappened
  const primarySaleHappened = beet.bool.read(buf, cursor);
  cursor += beet.bool.byteSize;

  // isMutable
  const isMutable = beet.bool.read(buf, cursor);
  cursor += beet.bool.byteSize;

  // editionNonce
  const editionNonceBeet = beet.coption(beet.u8).toFixedFromData(buf, cursor);
  const editionNonce = editionNonceBeet.read(buf, cursor);
  cursor += editionNonceBeet.byteSize;

  // -----------------
  // Possibly corrupted section
  // -----------------

  // NOTE: that we avoid trying to deserialize any subsequent fields if a
  // previous one was found to be corrupted just to save work

  // tokenStandard
  const [tokenStandard, tokenDelta, tokenCorrupted] = tryReadOption(
    beet.coption(tokenStandardBeet),
    buf,
    cursor,
  );
  cursor += tokenDelta;

  // collection
  const [collection, collectionDelta, collectionCorrupted] = tokenCorrupted
    ? [null, NONE_BYTE_SIZE, true]
    : tryReadOption(beet.coption(collectionBeet), buf, cursor);
  cursor += collectionDelta;

  // uses
  const [uses, usesDelta, usesCorrupted] =
    tokenCorrupted || collectionCorrupted
      ? [null, NONE_BYTE_SIZE, true]
      : tryReadOption(beet.coption(usesBeet), buf, cursor);
  cursor += usesDelta;

  // collection_details
  const [collectionDetails, collectionDetailsDelta, collectionDetailsCorrupted] =
    tokenCorrupted || collectionCorrupted || usesCorrupted
      ? [null, NONE_BYTE_SIZE, true]
      : tryReadOption(beet.coption(collectionDetailsBeet), buf, cursor);
  cursor += collectionDetailsDelta;

  const anyCorrupted =
    tokenCorrupted || collectionCorrupted || usesCorrupted || collectionDetailsCorrupted;

  const args = {
    key,
    updateAuthority,
    mint,
    data,
    primarySaleHappened,
    isMutable,
    editionNonce,
    tokenStandard: anyCorrupted ? null : tokenStandard,
    collection: anyCorrupted ? null : collection,
    uses: anyCorrupted ? null : uses,
    collectionDetails: anyCorrupted ? null : collectionDetails,
  };

  return [Metadata.fromArgs(args), cursor];
}

function tryReadOption<T>(
  optionBeet: beet.FixableBeet<T, Partial<T>>,
  buf: Buffer,
  offset: number,
): [T | null, number, boolean] {
  try {
    const fixed = optionBeet.toFixedFromData(buf, offset);
    const value = fixed.read(buf, offset);
    return [value, fixed.byteSize, false];
  } catch (err) {
    return [null, NONE_BYTE_SIZE, true];
  }
}
