import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { CandyMachineData } from '../types/CandyMachineData';
export declare type UpdateInstructionArgs = {
    data: CandyMachineData;
};
export declare const updateStruct: beet.FixableBeetArgsStruct<UpdateInstructionArgs & {
    instructionDiscriminator: number[];
}>;
export declare type UpdateInstructionAccounts = {
    candyMachine: web3.PublicKey;
    authority: web3.PublicKey;
    wallet: web3.PublicKey;
};
export declare const updateInstructionDiscriminator: number[];
export declare function createUpdateInstruction(accounts: UpdateInstructionAccounts, args: UpdateInstructionArgs, programId?: web3.PublicKey): web3.TransactionInstruction;
