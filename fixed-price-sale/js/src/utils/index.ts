import { TokenAccount } from '@metaplex-foundation/mpl-core';
import { Edition, MasterEdition, Metadata } from '@metaplex-foundation/mpl-token-metadata';
import { Connection, PublicKey } from '@solana/web3.js';
import { PROGRAM_ID } from '../consts';

const VAULT_OWNER_PREFIX = 'mt_vault';
const HISTORY_PREFIX = 'history';
const PAYOUT_TICKET_PREFIX = 'payout_ticket';
const HOLDER_PREFIX = 'holder';
const SECONDARY_METADATA_CREATORS_PREFIX = 'secondary_creators';

export const findVaultOwnerAddress = (
  mint: PublicKey,
  store: PublicKey,
): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [Buffer.from(VAULT_OWNER_PREFIX), mint.toBuffer(), store.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );

export const findTreasuryOwnerAddress = (
  treasuryMint: PublicKey,
  sellingResource: PublicKey,
): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [Buffer.from(HOLDER_PREFIX), treasuryMint.toBuffer(), sellingResource.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );

export const findTradeHistoryAddress = (
  wallet: PublicKey,
  market: PublicKey,
): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [Buffer.from(HISTORY_PREFIX), wallet.toBuffer(), market.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );

export const findPayoutTicketAddress = (
  funder: PublicKey,
  market: PublicKey,
): Promise<[PublicKey, number]> => {
  return PublicKey.findProgramAddress(
    [Buffer.from(PAYOUT_TICKET_PREFIX), funder.toBuffer(), market.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );
};

export const findSecondaryMetadataCreatorsAddress = (
  metadata: PublicKey,
): Promise<[PublicKey, number]> =>
  PublicKey.findProgramAddress(
    [Buffer.from(SECONDARY_METADATA_CREATORS_PREFIX), metadata.toBuffer()],
    new PublicKey(PROGRAM_ID),
  );

export const validateMembershipToken = async (
  connection: Connection,
  me: MasterEdition,
  ta: TokenAccount,
) => {
  const edition = (await Metadata.getEdition(connection, ta.data.mint)) as Edition;
  return edition.data.parent === me.pubkey.toString();
};
