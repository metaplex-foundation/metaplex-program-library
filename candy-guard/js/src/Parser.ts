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

/*
pub struct CandyGuardData {
    /// Last instruction check and bot tax (penalty for invalid transactions).
    pub bot_tax: Option<BotTax>,
    /// Live data guard (controls when minting is allowed).
    pub live_date: Option<LiveDate>,
    /// Lamports guard (set the price for the mint in lamports).
    pub lamports: Option<Lamports>,
    /// Spl-token guard (set the price for the mint in spl-token amount).
    pub spl_token: Option<SplToken>,
    /// Third party signer guard.
    pub third_party_signer: Option<ThirdPartySigner>,
    /// Whitelist guard (whitelist mint settings).
    pub whitelist: Option<Whitelist>,
    /// Gatekeeper guard
    pub gatekeeper: Option<Gatekeeper>,
    /// End settings guard
    pub end_settings: Option<EndSettings>,
}
*/

/**
 * Matching the guards of the related struct in the Rust program.
 * Make sure to update this whenever the Rust struct changes.
 * ```
 * pub struct CandyGuardData {
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
};
const GUARDS_COUNT = 9;

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
  };
}

export function parseData(buffer: Buffer): CandyGuardData {
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
  } = guards;
  logDebug('Guards: %O', guards);

  // data offset for deserialization (skip u64 features flag)
  let cursor = 8;
  // deserialized guards
  let data = {};

  if (botTaxEnabled) {
    const [botTax, offset] = botTaxBeet.deserialize(buffer, cursor);
    data['botTax'] = botTax;
    cursor = offset;
  }

  if (liveDateEnabled) {
    const [liveDate, offset] = liveDateBeet.deserialize(buffer, cursor);
    data['liveDate'] = liveDate;
    cursor = offset;
  }

  if (lamportsEnabled) {
    const [lamports, offset] = lamportsBeet.deserialize(buffer, cursor);
    data['lamports'] = lamports;
    cursor = offset;
  }

  if (splTokenEnabled) {
    const [splToken, offset] = splTokenBeet.deserialize(buffer, cursor);
    data['splToken'] = splToken;
    cursor = offset;
  }

  if (thirdPartySignerEnabled) {
    const [thirdPartySigner, offset] = thirdPartySignerBeet.deserialize(buffer, cursor);
    data['thirdPartySigner'] = thirdPartySigner;
    cursor = offset;
  }

  if (whitelistEnabled) {
    const [whitelist, offset] = whitelistBeet.deserialize(buffer, cursor);
    data['whitelist'] = whitelist;
    cursor = offset;
  }

  if (gatekeeperEnabled) {
    const [gatekeeper, offset] = gatekeeperBeet.deserialize(buffer, cursor);
    data['gatekeeper'] = gatekeeper;
    cursor = offset;
  }

  if (endSettingsEnabled) {
    const [endSettings, offset] = endSettingsBeet.deserialize(buffer, cursor);
    data['endSettings'] = endSettings;
    cursor = offset;
  }

  if (allowListEnabled) {
    const [allowList, offset] = allowListBeet.deserialize(buffer, cursor);
    data['allowList'] = allowList
    cursor = offset;
  }

  return {
    default: {
      botTax: data['botTax'] ?? null,
      liveDate: data['liveDate'] ?? null,
      lamports: data['lamports'] ?? null,
      splToken: data['splToken'] ?? null,
      thirdPartySigner: data['thirdPartySigner'] ?? null,
      whitelist: data['whitelist'] ?? null,
      gatekeeper: data['gateKeeper'] ?? null,
      endSettings: data['endSettings'] ?? null,
      allowList: data['allowList'] ?? null,
    },
    groups: null,
  };
}
