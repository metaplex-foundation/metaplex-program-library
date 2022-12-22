// @ts-ignore
import { encode } from '@msgpack/msgpack';
import { PublicKey } from '@solana/web3.js';

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
