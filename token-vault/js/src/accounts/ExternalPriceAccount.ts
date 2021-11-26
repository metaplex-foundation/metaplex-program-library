import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '@metaplex/errors';
import { AnyPublicKey, StringPublicKey } from '@metaplex/types';
import { Borsh } from '@metaplex/utils';
import { AccountInfo } from '@solana/web3.js';
import BN from 'bn.js';
import { Buffer } from 'buffer';
import { Account } from '../../../Account';
import { VaultKey, VaultProgram } from '../VaultProgram';

type Args = {
  pricePerShare: BN;
  priceMint: StringPublicKey;
  allowedToCombine: boolean;
};
export class ExternalPriceAccountData extends Borsh.Data<Args> {
  static readonly SCHEMA = this.struct([
    ['key', 'u8'],
    ['pricePerShare', 'u64'],
    ['priceMint', 'pubkeyAsString'],
    ['allowedToCombine', 'u8'],
  ]);

  key: VaultKey;
  pricePerShare: BN;
  /// Mint of the currency we are pricing the shares against, should be same as redeem_treasury.
  /// Most likely will be USDC mint most of the time.
  priceMint: StringPublicKey;
  /// Whether or not combination has been allowed for this vault.
  allowedToCombine: boolean;

  constructor(args: Args) {
    super(args);
    this.key = VaultKey.ExternalPriceAccountV1;
  }
}

export class ExternalPriceAccount extends Account<ExternalPriceAccountData> {
  constructor(key: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(key, info);

    if (!this.assertOwner(VaultProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (!ExternalPriceAccount.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = ExternalPriceAccountData.deserialize(this.info.data);
  }

  static isCompatible(data: Buffer) {
    return data[0] === VaultKey.ExternalPriceAccountV1;
  }
}
