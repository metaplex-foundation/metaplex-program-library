import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { ConfigLine } from '../types/ConfigLine';
export declare type AddConfigLinesInstructionArgs = {
    index: number;
    configLines: ConfigLine[];
};
export declare const addConfigLinesStruct: beet.FixableBeetArgsStruct<AddConfigLinesInstructionArgs & {
    instructionDiscriminator: number[];
}>;
export declare type AddConfigLinesInstructionAccounts = {
    candyMachine: web3.PublicKey;
    authority: web3.PublicKey;
};
export declare const addConfigLinesInstructionDiscriminator: number[];
export declare function createAddConfigLinesInstruction(accounts: AddConfigLinesInstructionAccounts, args: AddConfigLinesInstructionArgs, programId?: web3.PublicKey): web3.TransactionInstruction;
