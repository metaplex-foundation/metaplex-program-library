import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import bs58 from 'bs58';
import { AnyPublicKey, StringPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { Account } from '../../../Account';
import { MetaplexKey, MetaplexProgram } from '../MetaplexProgram';
import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '@metaplex/errors';
import { Buffer } from 'buffer';

type Args = { recipient: StringPublicKey; amountPaid: BN };
export class PayoutTicketData extends Borsh.Data<Args> {
  static readonly SCHEMA = this.struct([
    ['key', 'u8'],
    ['recipient', 'pubkeyAsString'],
    ['amountPaid', 'u64'],
  ]);

  key: MetaplexKey;
  recipient: StringPublicKey;
  amountPaid: BN;

  constructor(args: Args) {
    super(args);
    this.key = MetaplexKey.PayoutTicketV1;
  }
}

export class PayoutTicket extends Account<PayoutTicketData> {
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);

    if (!this.assertOwner(MetaplexProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (!PayoutTicket.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = PayoutTicketData.deserialize(this.info.data);
  }

  static isCompatible(data: Buffer) {
    return data[0] === MetaplexKey.PayoutTicketV1;
  }

  static async getPayoutTicketsByRecipient(connection: Connection, recipient: AnyPublicKey) {
    return (
      await MetaplexProgram.getProgramAccounts(connection, {
        filters: [
          // Filter for PayoutTicketV1 by key
          {
            memcmp: {
              offset: 0,
              bytes: bs58.encode(Buffer.from([MetaplexKey.PayoutTicketV1])),
            },
          },
          // Filter for assigned to recipient
          {
            memcmp: {
              offset: 1,
              bytes: new PublicKey(recipient).toBase58(),
            },
          },
        ],
      })
    ).map((account) => PayoutTicket.from(account));
  }
}
