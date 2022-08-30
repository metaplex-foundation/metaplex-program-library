import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { CandyGuardData } from '../types/CandyGuardData';
export declare type InitializeInstructionArgs = {
    data: CandyGuardData;
};
export declare const initializeStruct: beet.FixableBeetArgsStruct<InitializeInstructionArgs & {
    instructionDiscriminator: number[];
}>;
export declare type InitializeInstructionAccounts = {
    candyGuard: web3.PublicKey;
    base: web3.PublicKey;
    authority: web3.PublicKey;
    payer: web3.PublicKey;
    systemProgram?: web3.PublicKey;
};
export declare const initializeInstructionDiscriminator: number[];
export declare function createInitializeInstruction(accounts: InitializeInstructionAccounts, args: InitializeInstructionArgs, programId?: web3.PublicKey): web3.TransactionInstruction;
