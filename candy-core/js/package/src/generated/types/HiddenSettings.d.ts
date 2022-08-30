import * as beet from '@metaplex-foundation/beet';
export declare type HiddenSettings = {
    name: string;
    uri: string;
    hash: number[];
};
export declare const hiddenSettingsBeet: beet.FixableBeetArgsStruct<HiddenSettings>;
