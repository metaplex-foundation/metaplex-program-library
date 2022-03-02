import { AddressLabels } from '@metaplex-foundation/amman';
import { logDebug } from './log';

const persistLabelsPath = process.env.ADDRESS_LABEL_PATH;
const knownLabels = { ['metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s']: 'mpl-token-metadata' };

const addressLabels = new AddressLabels(knownLabels, logDebug, persistLabelsPath);
export const addLabel = addressLabels.addLabel;
export const isKeyOf = addressLabels.isKeyOf;
