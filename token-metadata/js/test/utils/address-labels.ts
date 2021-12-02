import { Keypair, PublicKey } from '@solana/web3.js';
import fs from 'fs';
import { logDebug } from '.';

const dataPath = process.env.ADDRESS_LABEL_PATH;
export const data = { ['metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s']: 'mpl-token-metadata' };

type Key = string | PublicKey | Keypair;
function publicKeyString(key: Key) {
  return typeof key === 'string'
    ? key
    : key instanceof Keypair
    ? key.publicKey.toBase58()
    : key.toBase58();
}

/**
 * Adds the key with the provided label to the known keys map.
 * This improves output of assertions and more.
 *
 * When the `ADDRESS_LABEL_PATH` env var is provided this writes a map of keypair:label entries
 * to the provided path in JSON format.
 * These can then be picked up by tools like the solana explorer in order to
 * render more meaningful labels of accounts.
 */
export function addLabel(label: string, key: Key) {
  logDebug(`🔑 ${label}: ${publicKeyString}`);
  const keyString = publicKeyString(key);

  if (dataPath == null) return;
  data[keyString] = label;
  fs.writeFileSync(dataPath, JSON.stringify(data, null, 2), 'utf8');
}

export function isKeyOf(key: Key) {
  const keyString = publicKeyString(key);
  const label = data[keyString];
  const fn = (otherKey: Key) => {
    const otherKeyString = publicKeyString(otherKey);
    return keyString === otherKeyString;
  };
  if (label != null) {
    fn.$spec = `isKeyOf('${label}')`;
  }
  return fn;
}
