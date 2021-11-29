import { PublicKey } from '@solana/web3.js';
import { config } from '../../config';
import { Program } from '../../Program';

export class AuctionProgram extends Program {
  static readonly PREFIX = 'auction';
  static readonly EXTENDED = 'extended';
  static readonly PUBKEY = new PublicKey(config.programs.auction);
}
