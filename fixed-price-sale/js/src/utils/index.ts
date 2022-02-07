import { PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from '../consts';

const VAULT_OWNER_PREFIX = 'mt_vault';
const HISTORY_PREFIX = 'history';
const PAYOUT_TICKET_PREFIX = 'payout_ticket';
const HOLDER_PREFIX = 'holder';

export const findVaultOwnerAddress = (mint: PublicKey, store: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from(VAULT_OWNER_PREFIX), mint.toBuffer(), store.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};

export const findTresuryOwnerAddress = (treasuryMint: PublicKey, sellingResource: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from(HOLDER_PREFIX), treasuryMint.toBuffer(), sellingResource.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};

export const findTradeHistoryAddress = (wallet: PublicKey, market: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from(HISTORY_PREFIX), wallet.toBuffer(), market.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};

export const findPayoutTicketAddress = (funder: PublicKey, market: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from(PAYOUT_TICKET_PREFIX), funder.toBuffer(), market.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};
