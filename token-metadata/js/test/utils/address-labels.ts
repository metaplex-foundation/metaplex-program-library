import { AddressLabels, KeyLike } from '@metaplex-foundation/amman';
import { logDebug } from '.';

const persistLabelsPath = process.env.ADDRESS_LABEL_PATH;
const knownLabels = { ['metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s']: 'mpl-token-metadata' };

export const addressLabels = new AddressLabels(knownLabels, logDebug, persistLabelsPath);

export function addLabel(label: string, key: KeyLike) {
  addressLabels.addLabel(label, key);
}

export function isKeyOf(key: KeyLike) {
  return addressLabels.isKeyOf(key);
}
