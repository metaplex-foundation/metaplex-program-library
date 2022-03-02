import { ERROR_INVALID_ACCOUNT_DATA, ERROR_INVALID_OWNER } from '../errors';
import { AnyPublicKey } from '../types';
import { Account } from './Account';
import {
  AccountInfo as TokenAccountInfo,
  AccountLayout,
  TOKEN_PROGRAM_ID,
  u64,
} from '@solana/spl-token';
import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import { Buffer } from 'buffer';

export class TokenAccount extends Account<TokenAccountInfo> {
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);

    if (!this.assertOwner(TOKEN_PROGRAM_ID)) {
      throw ERROR_INVALID_OWNER();
    }

    if (this.info == null || !TokenAccount.isCompatible(this.info.data)) {
      throw ERROR_INVALID_ACCOUNT_DATA();
    }

    this.data = deserialize(this.info.data);
  }

  static isCompatible(data: Buffer) {
    return data.length === AccountLayout.span;
  }

  static async getTokenAccountsByOwner(connection: Connection, owner: AnyPublicKey) {
    return (
      await connection.getTokenAccountsByOwner(new PublicKey(owner), {
        programId: TOKEN_PROGRAM_ID,
      })
    ).value.map(({ pubkey, account }) => new TokenAccount(pubkey, account));
  }
}

export const deserialize = (data: Buffer) => {
  const accountInfo = AccountLayout.decode(data);
  accountInfo.mint = new PublicKey(accountInfo.mint);
  accountInfo.owner = new PublicKey(accountInfo.owner);
  accountInfo.amount = u64.fromBuffer(accountInfo.amount);

  if (accountInfo.delegateOption === 0) {
    accountInfo.delegate = null;
    accountInfo.delegatedAmount = new u64(0);
  } else {
    accountInfo.delegate = new PublicKey(accountInfo.delegate);
    accountInfo.delegatedAmount = u64.fromBuffer(accountInfo.delegatedAmount);
  }

  accountInfo.isInitialized = accountInfo.state !== 0;
  accountInfo.isFrozen = accountInfo.state === 2;

  if (accountInfo.isNativeOption === 1) {
    accountInfo.rentExemptReserve = u64.fromBuffer(accountInfo.isNative);
    accountInfo.isNative = true;
  } else {
    accountInfo.rentExemptReserve = null;
    accountInfo.isNative = false;
  }

  if (accountInfo.closeAuthorityOption === 0) {
    accountInfo.closeAuthority = null;
  } else {
    accountInfo.closeAuthority = new PublicKey(accountInfo.closeAuthority);
  }

  return accountInfo;
};
