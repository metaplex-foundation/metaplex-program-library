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
} from './generated/types';
import { CandyGuard } from './generated/accounts/CandyGuard';
import { BN } from 'bn.js';

export function parseData(candyGuard: CandyGuard, buffer: Buffer): CandyGuardData {
  // enabled features from the candy guard account
  const enabled = new BN(candyGuard.features).toNumber();
  // data offset for deserialization
  let current = 0;
  // deserialized guards
  let data = {};

  // botTax
  let feature = 1;

  if (feature & enabled) {
    const [botTax, offset] = botTaxBeet.deserialize(buffer, current);
    data['botTax'] = botTax;
    current = offset;
  }

  // liveDate
  feature = feature << 1;

  if (feature & enabled) {
    const [liveDate, offset] = liveDateBeet.deserialize(buffer, current);
    data['liveDate'] = liveDate;
    current = offset;
  }

  // lamports
  feature = feature << 1;

  if (feature & enabled) {
    const [lamports, offset] = lamportsBeet.deserialize(buffer, current);
    data['lamports'] = lamports;
    current = offset;
  }

  // spl token
  feature = feature << 1;

  if (feature & enabled) {
    const [splToken, offset] = splTokenBeet.deserialize(buffer, current);
    data['splToken'] = splToken;
    current = offset;
  }

  // third party signer
  feature = feature << 1;

  if (feature & enabled) {
    const [thirdPartySigner, offset] = thirdPartySignerBeet.deserialize(buffer, current);
    data['thirdPartySigner'] = thirdPartySigner;
    current = offset;
  }

  // whitelist
  feature = feature << 1;

  if (feature & enabled) {
    const [whitelist, offset] = whitelistBeet.deserialize(buffer, current);
    data['whitelist'] = whitelist;
    current = offset;
  }

  // gatekeeper
  feature = feature << 1;

  if (feature & enabled) {
    const [gatekeeper, offset] = gatekeeperBeet.deserialize(buffer, current);
    data['gatekeeper'] = gatekeeper;
    current = offset;
  }

  // endSettings
  feature = feature << 1;

  if (feature & enabled) {
    const [endSettings, offset] = endSettingsBeet.deserialize(buffer, current);
    data['endSettings'] = endSettings;
    current = offset;
  }

  return {
    botTax: data['botTax'] ?? null,
    liveDate: data['liveDate'] ?? null,
    lamports: data['lamports'] ?? null,
    splToken: data['splToken'] ?? null,
    thirdPartySigner: data['thirdPartySigner'] ?? null,
    whitelist: data['whitelist'] ?? null,
    gatekeeper: data['gateKeeper'] ?? null,
    endSettings: data['endSettings'] ?? null,
  };
}
