import * as beet from '@metaplex-foundation/beet';
export declare type BotTax = {
    lamports: beet.bignum;
    lastInstruction: boolean;
};
export declare const botTaxBeet: beet.BeetArgsStruct<BotTax>;
