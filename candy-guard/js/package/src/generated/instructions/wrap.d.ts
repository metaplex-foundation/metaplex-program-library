import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
export declare const wrapStruct: beet.BeetArgsStruct<{
    instructionDiscriminator: number[];
}>;
export declare type WrapInstructionAccounts = {
    candyGuard: web3.PublicKey;
    candyMachine: web3.PublicKey;
    candyMachineProgram: web3.PublicKey;
    authority: web3.PublicKey;
};
export declare const wrapInstructionDiscriminator: number[];
export declare function createWrapInstruction(accounts: WrapInstructionAccounts, programId?: web3.PublicKey): web3.TransactionInstruction;
