import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
export declare type SetAuthorityInstructionArgs = {
    newAuthority: web3.PublicKey;
    newUpdateAuthority: web3.PublicKey;
};
export declare const setAuthorityStruct: beet.BeetArgsStruct<SetAuthorityInstructionArgs & {
    instructionDiscriminator: number[];
}>;
export declare type SetAuthorityInstructionAccounts = {
    candyMachine: web3.PublicKey;
    authority: web3.PublicKey;
};
export declare const setAuthorityInstructionDiscriminator: number[];
export declare function createSetAuthorityInstruction(accounts: SetAuthorityInstructionAccounts, args: SetAuthorityInstructionArgs, programId?: web3.PublicKey): web3.TransactionInstruction;
