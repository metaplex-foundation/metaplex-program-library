import {
  CandyGuardData,
  botTaxBeet,
  liveDateBeet,
  lamportsBeet,
  splTokenBeet,
  thirdPartySignerBeet,
  whitelistBeet,
  gatekeeperBeet,
  endSettingsBeet,
  allowListBeet,
} from './generated/types';
import { BN } from 'bn.js';
import * as beet from '@metaplex-foundation/beet';
import { logDebug } from './utils/log';
import { mintLimitBeet } from './generated/types/MintLimit';
import { GuardSet, guardSetBeet } from './generated/types/GuardSet';
import { nftPaymentBeet } from './generated/types/NftPayment';

/**
 * Matching the guards of the related struct in the Rust program.
 * Make sure to update this whenever the Rust struct changes.
 * ```
 * pub struct GuardSet {
 *     /// Last instruction check and bot tax (penalty for invalid transactions).
 *     pub bot_tax: Option<BotTax>,
 *     /// Live data guard (controls when minting is allowed).
 *     pub live_date: Option<LiveDate>,
 *     /// Lamports guard (set the price for the mint in lamports).
 *     pub lamports: Option<Lamports>,
 *     /// Spl-token guard (set the price for the mint in spl-token amount).
 *     pub spl_token: Option<SplToken>,
 *     /// Third party signer guard.
 *     pub third_party_signer: Option<ThirdPartySigner>,
 *     /// Whitelist guard (whitelist mint settings).
 *     pub whitelist: Option<Whitelist>,
 *     /// Gatekeeper guard
 *     pub gatekeeper: Option<Gatekeeper>,
 *     /// End settings guard
 *     pub end_settings: Option<EndSettings>,
 *     /// Allow list guard
 *     pub allow_list: Option<AllowList>,
 *     /// Mint limit guard
 *     pub mint_limit: Option<MintLimit>,
 * }
 * ```
 */

type Guards = {
  /* 01 */ botTaxEnabled: boolean;
  /* 02 */ liveDateEnabled: boolean;
  /* 01 */ lamportsEnabled: boolean;
  /* 04 */ splTokenEnabled: boolean;
  /* 05 */ thirdPartySignerEnabled: boolean;
  /* 06 */ whitelistEnabled: boolean;
  /* 07 */ gatekeeperEnabled: boolean;
  /* 08 */ endSettingsEnabled: boolean;
  /* 09 */ allowListEnabled: boolean;
  /* 10 */ mintLimitEnabled: boolean;
  /* 11 */ nftPaymentEnabled: boolean;
};

const GUARDS_SIZE = {
  /* 01 */ botTax: 9,
  /* 02 */ liveDate: 9,
  /* 01 */ lamports: 8,
  /* 04 */ splToken: 40,
  /* 05 */ thirdPartySigner: 32,
  /* 06 */ whitelist: 43,
  /* 07 */ gatekeeper: 33,
  /* 08 */ endSettings: 9,
  /* 09 */ allowList: 32,
  /* 10 */ mintLimit: 5,
  /* 11 */ nftPayment: 33,
};
const GUARDS_COUNT = 11;

function determineGuards(buffer: Buffer): Guards {
  const enabled = new BN(beet.u64.read(buffer, 0)).toNumber();

  const guards: boolean[] = [];
  for (let i = 0; i < GUARDS_COUNT; i++) {
    guards.push(!!((1 << i) & enabled));
  }

  const [
    botTaxEnabled,
    liveDateEnabled,
    lamportsEnabled,
    splTokenEnabled,
    thirdPartySignerEnabled,
    whitelistEnabled,
    gatekeeperEnabled,
    endSettingsEnabled,
    allowListEnabled,
    mintLimitEnabled,
    nftPaymentEnabled,
  ] = guards;

  return {
    botTaxEnabled,
    liveDateEnabled,
    lamportsEnabled,
    splTokenEnabled,
    thirdPartySignerEnabled,
    whitelistEnabled,
    gatekeeperEnabled,
    endSettingsEnabled,
    allowListEnabled,
    mintLimitEnabled,
    nftPaymentEnabled,
  };
}

export function parseData(buffer: Buffer): CandyGuardData {
  // parses the default guard set
  const { guardSet: defaultSet, offset } = parseGuardSet(buffer);
  // retrieves the number of groups
  const groupsCount = new BN(beet.u32.read(buffer, offset)).toNumber();
  const groups: GuardSet[] = [];

  let cursor = beet.u32.byteSize + offset;
  for (let i = 0; i < groupsCount; i++) {
    // parses each individual guard set
    const { guardSet: group, offset } = parseGuardSet(buffer.subarray(cursor));
    groups.push(group);
    cursor += offset;
  }

  return {
    default: defaultSet,
    groups: groups.length === 0 ? null : groups,
  };
}

function parseGuardSet(buffer: Buffer): { guardSet: GuardSet; offset: number } {
  const guards = determineGuards(buffer);
  const {
    botTaxEnabled,
    liveDateEnabled,
    lamportsEnabled,
    splTokenEnabled,
    thirdPartySignerEnabled,
    whitelistEnabled,
    gatekeeperEnabled,
    endSettingsEnabled,
    allowListEnabled,
    mintLimitEnabled,
    nftPaymentEnabled,
  } = guards;
  logDebug('Guards: %O', guards);

  // data offset for deserialization (skip u64 features flag)
  let cursor = beet.u64.byteSize;
  // deserialized guards
  const data: Record<string, any> = {};

  if (botTaxEnabled) {
    const [botTax] = botTaxBeet.deserialize(buffer, cursor);
    data.botTax = botTax;
    cursor += GUARDS_SIZE.botTax;
  }

  if (liveDateEnabled) {
    const [liveDate] = liveDateBeet.deserialize(buffer, cursor);
    data.liveDate = liveDate;
    cursor += GUARDS_SIZE.liveDate;
  }

  if (lamportsEnabled) {
    const [lamports] = lamportsBeet.deserialize(buffer, cursor);
    data.lamports = lamports;
    cursor += GUARDS_SIZE.lamports;
  }

  if (splTokenEnabled) {
    const [splToken] = splTokenBeet.deserialize(buffer, cursor);
    data.splToken = splToken;
    cursor += GUARDS_SIZE.splToken;
  }

  if (thirdPartySignerEnabled) {
    const [thirdPartySigner] = thirdPartySignerBeet.deserialize(buffer, cursor);
    data.thirdPartySigner = thirdPartySigner;
    cursor += GUARDS_SIZE.thirdPartySigner;
  }

  if (whitelistEnabled) {
    const [whitelist] = whitelistBeet.deserialize(buffer, cursor);
    data.whitelist = whitelist;
    cursor += GUARDS_SIZE.whitelist;
  }

  if (gatekeeperEnabled) {
    const [gatekeeper] = gatekeeperBeet.deserialize(buffer, cursor);
    data.gatekeeper = gatekeeper;
    cursor += GUARDS_SIZE.gatekeeper;
  }

  if (endSettingsEnabled) {
    const [endSettings] = endSettingsBeet.deserialize(buffer, cursor);
    data.endSettings = endSettings;
    cursor += GUARDS_SIZE.endSettings;
  }

  if (allowListEnabled) {
    const [allowList] = allowListBeet.deserialize(buffer, cursor);
    data.allowList = allowList;
    cursor += GUARDS_SIZE.allowList;
  }

  if (mintLimitEnabled) {
    const [mintLimit] = mintLimitBeet.deserialize(buffer, cursor);
    data.mintLimit = mintLimit;
    cursor += GUARDS_SIZE.mintLimit;
  }

  if (nftPaymentEnabled) {
    const [nftPayment] = nftPaymentBeet.deserialize(buffer, cursor);
    data.nftPayment = nftPayment;
    cursor += GUARDS_SIZE.nftPayment;
  }

  return {
    guardSet: {
      botTax: data.botTax ?? null,
      liveDate: data.liveDate ?? null,
      lamports: data.lamports ?? null,
      splToken: data.splToken ?? null,
      thirdPartySigner: data.thirdPartySigner ?? null,
      whitelist: data.whitelist ?? null,
      gatekeeper: data.gateKeeper ?? null,
      endSettings: data.endSettings ?? null,
      allowList: data.allowList ?? null,
      mintLimit: data.mintLimit ?? null,
      nftPayment: data.nftPayment ?? null,
    },
    offset: cursor,
  };
}
