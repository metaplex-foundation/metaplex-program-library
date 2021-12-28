import React, { useCallback, useContext, useEffect, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import { AccountLayout, MintInfo, u64 } from '@solana/spl-token';
import { WRAPPED_SOL_MINT } from '@project-serum/serum/lib/token-instructions';

import { useConnection } from '../ConnectionContext';
import { TokenAccount } from '../../models';
import { genericCache, cache } from './cache';
import { deserializeAccount } from './deserialize';
import { TokenAccountParser, MintParser } from './parsesrs';

const AccountsContext = React.createContext<any>(null);

export const useAccountsContext = () => {
  const context = useContext(AccountsContext);

  return context;
};

function wrapNativeAccount(
  pubkey: PublicKey,
  account?: AccountInfo<Buffer>,
): TokenAccount | undefined {
  if (!account) {
    return undefined;
  }

  return {
    pubkey: pubkey.toBase58(),
    account,
    info: {
      address: pubkey,
      mint: WRAPPED_SOL_MINT,
      owner: pubkey,
      amount: new u64(account.lamports),
      delegate: null,
      delegatedAmount: new u64(0),
      isInitialized: true,
      isFrozen: false,
      isNative: true,
      rentExemptReserve: null,
      closeAuthority: null,
    },
  };
}

export const useNativeAccount = () => {
  const connection = useConnection();
  const { publicKey } = useWallet();

  const [nativeAccount, setNativeAccount] = useState<AccountInfo<Buffer>>();

  const updateCache = useCallback(
    account => {
      if (publicKey) {
        const wrapped = wrapNativeAccount(publicKey, account);
        if (wrapped !== undefined) {
          const id = publicKey.toBase58();
          cache.registerParser(id, TokenAccountParser);
          genericCache.set(id, wrapped as TokenAccount);
          cache.emitter.raiseCacheUpdated(id, false, TokenAccountParser, true);
        }
      }
    },
    [publicKey],
  );

  useEffect(() => {
    let subId = 0;
    const updateAccount = (account: AccountInfo<Buffer> | null) => {
      if (account) {
        updateCache(account);
        setNativeAccount(account);
      }
    };

    (async () => {
      if (!connection || !publicKey) {
        return;
      }

      const account = await connection.getAccountInfo(publicKey);
      updateAccount(account);

      subId = connection.onAccountChange(publicKey, updateAccount);
    })();

    return () => {
      if (subId) {
        connection.removeAccountChangeListener(subId);
      }
    };
  }, [setNativeAccount, publicKey, connection, updateCache]);

  return { account: nativeAccount };
};

