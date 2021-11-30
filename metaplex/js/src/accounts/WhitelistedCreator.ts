import { AnyPublicKey, StringPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { MetaplexProgram, MetaplexKey } from '../MetaplexProgram';
import { AccountInfo, PublicKey } from '@solana/web3.js';
import { Account } from '../../../Account';
import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '@metaplex/errors';
import { Buffer } from 'buffer';

type Args = { address: string; activated: boolean };
export class WhitelistedCreatorData extends Borsh.Data<Args> {
  static readonly SCHEMA = this.struct([
    ['key', 'u8'],
    ['address', 'pubkeyAsString'],
    ['activated', 'u8'],
  ]);

  key: MetaplexKey = MetaplexKey.WhitelistedCreatorV1;
  address: StringPublicKey;
  activated = true;

  // Populated from name service
  twitter?: string;
  name?: string;
  image?: string;
  description?: string;

  constructor(args: Args) {
    super(args);
    this.key = MetaplexKey.WhitelistedCreatorV1;
  }
}

export class WhitelistedCreator extends Account<WhitelistedCreatorData> {
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);

    if (!this.assertOwner(MetaplexProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (!WhitelistedCreator.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = WhitelistedCreatorData.deserialize(this.info.data);
  }

  static isCompatible(data: Buffer) {
    return data[0] === MetaplexKey.WhitelistedCreatorV1;
  }

  static async getPDA(store: AnyPublicKey, creator: AnyPublicKey) {
    return MetaplexProgram.findProgramAddress([
      Buffer.from(MetaplexProgram.PREFIX),
      MetaplexProgram.PUBKEY.toBuffer(),
      new PublicKey(store).toBuffer(),
      new PublicKey(creator).toBuffer(),
    ]);
  }
}
