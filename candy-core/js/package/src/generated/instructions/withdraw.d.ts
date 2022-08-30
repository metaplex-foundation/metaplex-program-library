import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
export declare const withdrawStruct: beet.BeetArgsStruct<{
    instructionDiscriminator: number[];
}>;
export declare type WithdrawInstructionAccounts = {
    candyMachine: web3.PublicKey;
    authority: web3.PublicKey;
};
export declare const withdrawInstructionDiscriminator: number[];
export declare function createWithdrawInstruction(accounts: WithdrawInstructionAccounts, programId?: web3.PublicKey): web3.TransactionInstruction;
