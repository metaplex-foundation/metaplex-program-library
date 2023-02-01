import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';
import { keyBeet } from '../generated/types/Key';
import { TokenRecord } from '../generated/accounts/TokenRecord';
import { tokenDelegateRoleBeet, tokenStateBeet } from 'src/generated';
import { tryReadOption } from '.';

/**
 * This is a custom deserializer for TokenRecord in order to support variable account sizes.
 */
export function deserialize(buf: Buffer, offset = 0): [TokenRecord, number] {
  let cursor = offset;

  // key
  const key = keyBeet.read(buf, cursor);
  cursor += keyBeet.byteSize;

  // updateAuthority
  const bump = beet.u8.read(buf, cursor);
  cursor += beet.u8.byteSize;

  // state
  const state = tokenStateBeet.read(buf, cursor);
  cursor += tokenStateBeet.byteSize;

  // ruleSetRevision
  const [ruleSetRevision, ruleSetRevisionDelta] = tryReadOption(
    beet.coption(beet.u64),
    buf,
    cursor,
  );
  cursor += ruleSetRevisionDelta;

  // delegate
  const [delegate, delegateDelta] = tryReadOption(beet.coption(beetSolana.publicKey), buf, cursor);
  cursor += delegateDelta;

  // delegateRole
  const [delegateRole, delegateRoleDelta] = tryReadOption(
    beet.coption(tokenDelegateRoleBeet),
    buf,
    cursor,
  );
  cursor += delegateRoleDelta;

  // lockedTransfer (could be missing)
  const [lockedTransfer, lockedTransferDelta, lockedTransferCorrupted] = tryReadOption(
    beet.coption(beetSolana.publicKey),
    buf,
    cursor,
  );
  cursor += lockedTransferDelta;

  const args = {
    key,
    bump,
    state,
    ruleSetRevision,
    delegate,
    delegateRole,
    lockedTransfer: lockedTransferCorrupted ? null : lockedTransfer,
  };

  return [TokenRecord.fromArgs(args), cursor];
}
