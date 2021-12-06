import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import bs58 from 'bs58';
import {
  Account,
  AnyPublicKey,
  Borsh,
  ERROR_INVALID_ACCOUNT_DATA,
  ERROR_INVALID_OWNER,
  StringPublicKey,
} from '@metaplex-foundation/mpl-core';
import { SafetyDepositBox } from './SafetyDepositBox';
import { VaultKey, VaultProgram } from '../VaultProgram';
import { Buffer } from 'buffer';

export class AmountArgs extends Borsh.Data<{
  instruction: number;
  amount: BN;
}> {
  static readonly SCHEMA = this.struct([
    ['instruction', 'u8'],
    ['amount', 'u64'],
  ]);

  instruction: number;
  amount: BN;
}

export class NumberOfShareArgs extends Borsh.Data<{
  instruction: number;
  numberOfShares: BN;
}> {
  static readonly SCHEMA = this.struct([
    ['instruction', 'u8'],
    ['numberOfShares', 'u64'],
  ]);

  instruction: number;
  numberOfShares: BN;
}

export enum VaultState {
  Inactive = 0,
  Active = 1,
  Combined = 2,
  Deactivated = 3,
}

type Args = {
  tokenProgram: StringPublicKey;
  fractionMint: StringPublicKey;
  authority: StringPublicKey;
  fractionTreasury: StringPublicKey;
  redeemTreasury: StringPublicKey;
  allowFurtherShareCreation: boolean;
  pricingLookupAddress: StringPublicKey;
  tokenTypeCount: number;
  state: VaultState;
  lockedPricePerShare: BN;
};
export class VaultData extends Borsh.Data<Args> {
  static readonly SCHEMA = this.struct([
    ['key', 'u8'],
    ['tokenProgram', 'pubkeyAsString'],
    ['fractionMint', 'pubkeyAsString'],
    ['authority', 'pubkeyAsString'],
    ['fractionTreasury', 'pubkeyAsString'],
    ['redeemTreasury', 'pubkeyAsString'],
    ['allowFurtherShareCreation', 'u8'],
    ['pricingLookupAddress', 'pubkeyAsString'],
    ['tokenTypeCount', 'u8'],
    ['state', 'u8'],
    ['lockedPricePerShare', 'u64'],
  ]);

  key: VaultKey;
  /// Store token program used
  tokenProgram: StringPublicKey;
  /// Mint that produces the fractional shares
  fractionMint: StringPublicKey;
  /// Authority who can make changes to the vault
  authority: StringPublicKey;
  /// treasury where fractional shares are held for redemption by authority
  fractionTreasury: StringPublicKey;
  /// treasury where monies are held for fractional share holders to redeem(burn) shares once buyout is made
  redeemTreasury: StringPublicKey;
  /// Can authority mint more shares from fraction_mint after activation
  allowFurtherShareCreation: boolean;

  /// Must point at an ExternalPriceAccount, which gives permission and price for buyout.
  pricingLookupAddress: StringPublicKey;
  /// In inactive state, we use this to set the order key on Safety Deposit Boxes being added and
  /// then we increment it and save so the next safety deposit box gets the next number.
  /// In the Combined state during token redemption by authority, we use it as a decrementing counter each time
  /// The authority of the vault withdrawals a Safety Deposit contents to count down how many
  /// are left to be opened and closed down. Once this hits zero, and the fraction mint has zero shares,
  /// then we can deactivate the vault.
  tokenTypeCount: number;
  state: VaultState;

  /// Once combination happens, we copy price per share to vault so that if something nefarious happens
  /// to external price account, like price change, we still have the math 'saved' for use in our calcs
  lockedPricePerShare: BN;

  constructor(args: Args) {
    super(args);
    this.key = VaultKey.VaultV1;
  }
}

export class Vault extends Account<VaultData> {
  static MAX_VAULT_SIZE = 1 + 32 + 32 + 32 + 32 + 1 + 32 + 1 + 32 + 1 + 1 + 8;
  static MAX_EXTERNAL_ACCOUNT_SIZE = 1 + 8 + 32 + 1;

  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);

    if (!this.assertOwner(VaultProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }

    if (!Vault.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = VaultData.deserialize(this.info.data);
  }

  static async getPDA(pubkey: AnyPublicKey) {
    return VaultProgram.findProgramAddress([
      Buffer.from(VaultProgram.PREFIX),
      VaultProgram.PUBKEY.toBuffer(),
      new PublicKey(pubkey).toBuffer(),
    ]);
  }

  static isCompatible(data: Buffer) {
    return data[0] === VaultKey.VaultV1;
  }

  async getSafetyDepositBoxes(connection: Connection) {
    return (
      await VaultProgram.getProgramAccounts(connection, {
        filters: [
          // Filter for SafetyDepositBoxV1 by key
          {
            memcmp: {
              offset: 0,
              bytes: bs58.encode(Buffer.from([VaultKey.SafetyDepositBoxV1])),
            },
          },
          // Filter for assigned to this vault
          {
            memcmp: {
              offset: 1,
              bytes: this.pubkey.toBase58(),
            },
          },
        ],
      })
    ).map((account) => SafetyDepositBox.from(account));
  }
}
