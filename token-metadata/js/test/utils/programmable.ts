// @ts-ignore
import { encode } from '@msgpack/msgpack';
import { PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from '../../src/generated';

export function createPassRuleSet(
  ruleSetName: string,
  owner: PublicKey,
  operation: string,
): Uint8Array {
  const operations = {};
  operations[operation] = 'Pass';

  const ruleSet = {
    ruleSetName,
    owner: Array.from(owner.toBytes()),
    operations,
  };
  return encode(ruleSet);
}

export function findTokenRecordPda(mint: PublicKey, token: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from('metadata'),
      PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from('token_record'),
      token.toBuffer(),
    ],
    PROGRAM_ID,
  )[0];
}
