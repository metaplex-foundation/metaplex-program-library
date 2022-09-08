import { initCusper } from '@metaplex-foundation/cusper';
import { errorFromCode } from '../../src/generated';

export const cusper = initCusper(errorFromCode);
