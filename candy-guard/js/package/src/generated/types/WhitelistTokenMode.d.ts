import * as beet from '@metaplex-foundation/beet';
export declare enum WhitelistTokenMode {
    BurnEveryTime = 0,
    NeverBurn = 1
}
export declare const whitelistTokenModeBeet: beet.FixedSizeBeet<WhitelistTokenMode, WhitelistTokenMode>;
