import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { CandyGuardData } from '../types/CandyGuardData';
export declare type UpdateInstructionArgs = {
    data: CandyGuardData;
};
export declare const updateStruct: beet.FixableBeetArgsStruct<UpdateInstructionArgs & {
    instructionDiscriminator: number[];
}>;
export declare type UpdateInstructionAccounts = {
    candyGuard: web3.PublicKey;
    authority: web3.PublicKey;
    payer: web3.PublicKey;
    systemProgram?: web3.PublicKey;
};
export declare const updateInstructionDiscriminator: number[];
export declare function createUpdateInstruction(accounts: UpdateInstructionAccounts, args: UpdateInstructionArgs, programId?: web3.PublicKey): web3.TransactionInstruction;
