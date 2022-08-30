import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
export declare const unwrapStruct: beet.BeetArgsStruct<{
    instructionDiscriminator: number[];
}>;
export declare type UnwrapInstructionAccounts = {
    candyGuard: web3.PublicKey;
    candyMachine: web3.PublicKey;
    candyMachineProgram: web3.PublicKey;
    authority: web3.PublicKey;
};
export declare const unwrapInstructionDiscriminator: number[];
export declare function createUnwrapInstruction(accounts: UnwrapInstructionAccounts, programId?: web3.PublicKey): web3.TransactionInstruction;
