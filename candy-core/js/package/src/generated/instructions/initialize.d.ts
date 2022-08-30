import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { CandyMachineData } from '../types/CandyMachineData';
export declare type InitializeInstructionArgs = {
    data: CandyMachineData;
};
export declare const initializeStruct: beet.FixableBeetArgsStruct<InitializeInstructionArgs & {
    instructionDiscriminator: number[];
}>;
export declare type InitializeInstructionAccounts = {
    candyMachine: web3.PublicKey;
    wallet: web3.PublicKey;
    authority: web3.PublicKey;
    updateAuthority: web3.PublicKey;
    payer: web3.PublicKey;
    systemProgram?: web3.PublicKey;
    rent?: web3.PublicKey;
};
export declare const initializeInstructionDiscriminator: number[];
export declare function createInitializeInstruction(accounts: InitializeInstructionAccounts, args: InitializeInstructionArgs, programId?: web3.PublicKey): web3.TransactionInstruction;
