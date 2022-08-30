import * as beet from '@metaplex-foundation/beet';
export declare type ConfigLineSettings = {
    prefixName: string;
    nameLength: number;
    prefixUri: string;
    uriLength: number;
    isSequential: boolean;
};
export declare const configLineSettingsBeet: beet.FixableBeetArgsStruct<ConfigLineSettings>;
