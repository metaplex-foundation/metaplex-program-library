export * from './CandyMachine';
export * from './CollectionPDA';
export * from './LockupSettings';

import { CandyMachine } from './CandyMachine';
import { CollectionPDA } from './CollectionPDA';
import { LockupSettings } from './LockupSettings';

export const accountProviders = { CandyMachine, CollectionPDA, LockupSettings };
