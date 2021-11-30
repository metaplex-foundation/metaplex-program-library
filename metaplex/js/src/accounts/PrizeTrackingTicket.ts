import { AccountInfo, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { AnyPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { Account } from '../../../Account';
import { MetaplexKey, MetaplexProgram } from '../MetaplexProgram';
import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '@metaplex/errors';
import { Buffer } from 'buffer';

type Args = {
  metadata: string;
  supplySnapshot: BN;
  expectedRedemptions: BN;
  redemptions: BN;
};
export class PrizeTrackingTicketData extends Borsh.Data<Args> {
  static readonly SCHEMA = this.struct([
    ['key', 'u8'],
    ['metadata', 'pubkeyAsString'],
    ['supplySnapshot', 'u64'],
    ['expectedRedemptions', 'u64'],
    ['redemptions', 'u64'],
  ]);

  key: MetaplexKey = MetaplexKey.PrizeTrackingTicketV1;
  metadata: string;
  supplySnapshot: BN;
  expectedRedemptions: BN;
  redemptions: BN;

  constructor(args: Args) {
    super(args);
    this.key = MetaplexKey.PrizeTrackingTicketV1;
  }
}

export class PrizeTrackingTicket extends Account<PrizeTrackingTicketData> {
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);

    if (!this.assertOwner(MetaplexProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (!PrizeTrackingTicket.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = PrizeTrackingTicketData.deserialize(this.info.data);
  }

  static isCompatible(data: Buffer) {
    return data[0] === MetaplexKey.PrizeTrackingTicketV1;
  }

  static async getPDA(auctionManager: AnyPublicKey, mint: AnyPublicKey) {
    return MetaplexProgram.findProgramAddress([
      Buffer.from(MetaplexProgram.PREFIX),
      MetaplexProgram.PUBKEY.toBuffer(),
      new PublicKey(auctionManager).toBuffer(),
      new PublicKey(mint).toBuffer(),
    ]);
  }
}
