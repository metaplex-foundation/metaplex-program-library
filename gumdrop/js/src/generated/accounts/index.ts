export * from './CandyMachine';
export * from './ClaimCount';
export * from './ClaimProof';
export * from './ClaimStatus';
export * from './Config';
export * from './MerkleDistributor';

import { MerkleDistributor } from './MerkleDistributor';
import { ClaimStatus } from './ClaimStatus';
import { ClaimCount } from './ClaimCount';
import { ClaimProof } from './ClaimProof';
import { CandyMachine } from './CandyMachine';
import { Config } from './Config';

export const accountProviders = {
  MerkleDistributor,
  ClaimStatus,
  ClaimCount,
  ClaimProof,
  CandyMachine,
  Config,
};
