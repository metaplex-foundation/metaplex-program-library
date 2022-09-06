export * from './CandyMachine';
export * from './CollectionPDA';
export * from './FreezePDA';

import { CandyMachine } from './CandyMachine';
import { CollectionPDA } from './CollectionPDA';
import { FreezePDA } from './FreezePDA';

export const accountProviders = { CandyMachine, CollectionPDA, FreezePDA };
