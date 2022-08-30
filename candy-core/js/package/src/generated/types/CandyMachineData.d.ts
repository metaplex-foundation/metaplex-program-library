import * as beet from '@metaplex-foundation/beet';
import { Creator } from './Creator';
import { ConfigLineSettings } from './ConfigLineSettings';
import { HiddenSettings } from './HiddenSettings';
export declare type CandyMachineData = {
    itemsAvailable: beet.bignum;
    symbol: string;
    sellerFeeBasisPoints: number;
    maxSupply: beet.bignum;
    isMutable: boolean;
    retainAuthority: boolean;
    creators: Creator[];
    configLineSettings: beet.COption<ConfigLineSettings>;
    hiddenSettings: beet.COption<HiddenSettings>;
};
export declare const candyMachineDataBeet: beet.FixableBeetArgsStruct<CandyMachineData>;
