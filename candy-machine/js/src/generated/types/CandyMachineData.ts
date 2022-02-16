import * as beet from '@metaplex-foundation/beet';
import * as definedTypes from '../types';
export type CandyMachineData = {
  uuid: string;
  price: beet.bignum;
  symbol: string;
  sellerFeeBasisPoints: number;
  maxSupply: beet.bignum;
  isMutable: boolean;
  retainAuthority: boolean;
  goLiveDate: beet.COption<beet.bignum>;
  endSettings: beet.COption<definedTypes.EndSettings>;
  creators: definedTypes.Creator[];
  hiddenSettings: beet.COption<definedTypes.HiddenSettings>;
  whitelistMintSettings: beet.COption<definedTypes.WhitelistMintSettings>;
  itemsAvailable: beet.bignum;
  gatekeeper: beet.COption<definedTypes.GatekeeperConfig>;
};
export const candyMachineDataStruct = new beet.FixableBeetArgsStruct<CandyMachineData>(
  [
    ['uuid', beet.utf8String],
    ['price', beet.u64],
    ['symbol', beet.utf8String],
    ['sellerFeeBasisPoints', beet.u16],
    ['maxSupply', beet.u64],
    ['isMutable', beet.bool],
    ['retainAuthority', beet.bool],
    ['goLiveDate', beet.coption(beet.i64)],
    ['endSettings', beet.coption(definedTypes.endSettingsStruct)],
    ['creators', beet.array(definedTypes.creatorStruct)],
    ['hiddenSettings', beet.coption(definedTypes.hiddenSettingsStruct)],
    ['whitelistMintSettings', beet.coption(definedTypes.whitelistMintSettingsStruct)],
    ['itemsAvailable', beet.u64],
    ['gatekeeper', beet.coption(definedTypes.gatekeeperConfigStruct)],
  ],
  'CandyMachineData',
);
