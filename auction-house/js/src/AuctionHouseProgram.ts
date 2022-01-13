import { config, Program } from '@metaplex-foundation/mpl-core';
import { PublicKey } from '@solana/web3.js';
import * as errors from './generated/errors';
import * as instructions from './generated/instructions';
import * as accounts from './generated/accounts';

export class AuctionHouseProgram extends Program {
  static readonly PREFIX = 'metaplex';
  static readonly CONFIG = 'config';
  static readonly TOTALS = 'totals';
  static readonly PUBKEY = new PublicKey(config.programs.auctionHouse);
  static readonly instructions = instructions;
  static readonly errors = errors;
  static readonly accounts = accounts;
}
