import { AddressLabels } from '@metaplex-foundation/amman';
import { PROGRAM_ID } from '../../src/consts';
import { logDebug } from '.';

const persistLabelsPath = process.env.ADDRESS_LABEL_PATH;
const knownLabels = {
  ['metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s']: 'mpl-token-metadata',
  [PROGRAM_ID]: 'mpl-membership-token',
};

const addressLabels = new AddressLabels(knownLabels, logDebug, persistLabelsPath);
export const addLabel = addressLabels.addLabel;
export const isKeyOf = addressLabels.isKeyOf;
