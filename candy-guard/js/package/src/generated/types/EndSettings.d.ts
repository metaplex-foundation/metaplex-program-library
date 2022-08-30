import * as beet from '@metaplex-foundation/beet';
import { EndSettingType } from './EndSettingType';
export declare type EndSettings = {
    endSettingType: EndSettingType;
    number: beet.bignum;
};
export declare const endSettingsBeet: beet.BeetArgsStruct<EndSettings>;
