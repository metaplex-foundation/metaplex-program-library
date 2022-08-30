/// <reference types="node" />
import { CandyGuardData } from './generated/types';
import { CandyGuard } from './generated/accounts/CandyGuard';
export declare function parseData(candyGuard: CandyGuard, buffer: Buffer): CandyGuardData;
