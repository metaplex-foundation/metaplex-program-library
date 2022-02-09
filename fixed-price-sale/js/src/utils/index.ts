import { TokenAccount } from '@metaplex-foundation/mpl-core';
import { Edition, MasterEdition, Metadata } from '@metaplex-foundation/mpl-token-metadata';
import { Connection, PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from '../consts';

const VAULT_OWNER_PREFIX = 'mt_vault';
const TREASURY_OWNER_PREFIX = 'holder';
const TRADE_HISTORY_PREFIX = 'history';

export const findVaultOwnerAddress = (mint: PublicKey, store: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from(VAULT_OWNER_PREFIX), mint.toBuffer(), store.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};

export const findTresuryOwnerAddress = (treasuryMint: PublicKey, sellingResource: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from(TREASURY_OWNER_PREFIX), treasuryMint.toBuffer(), sellingResource.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};

export const findTradeHistoryAddress = (wallet: PublicKey, market: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from(TRADE_HISTORY_PREFIX), wallet.toBuffer(), market.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};

export const validateMembershipToken = async (
  connection: Connection,
  me: MasterEdition,
  ta: TokenAccount,
) => {
  const edition = (await Metadata.getEdition(connection, ta.data.mint)) as Edition;
  return edition.data.parent === me.pubkey.toString();
};
