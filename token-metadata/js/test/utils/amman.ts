import { Amman } from '@metaplex-foundation/amman';
import { METADATA_PROGRAM_ADDRESS } from '../../src/common/consts';
import { logDebug } from './log';

export const amman = Amman.instance({
  knownLabels: { [METADATA_PROGRAM_ADDRESS]: 'mpl-token-metadata' },
  log: logDebug,
});
