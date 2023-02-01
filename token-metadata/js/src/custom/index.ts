import * as beet from '@metaplex-foundation/beet';

const NONE_BYTE_SIZE = beet.coptionNone('').byteSize;

export function tryReadOption<T>(
  optionBeet: beet.FixableBeet<T, Partial<T>>,
  buf: Buffer,
  offset: number,
): [T | null, number, boolean] {
  if (buf.subarray(offset).length == 0) {
    return [null, NONE_BYTE_SIZE, true];
  }

  const fixed = optionBeet.toFixedFromData(buf, offset);
  const value = fixed.read(buf, offset);
  return [value, fixed.byteSize, false];
}
