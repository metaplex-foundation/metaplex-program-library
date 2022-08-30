import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
export declare type MintInstructionArgs = {
    creatorBump: number;
};
export declare const mintStruct: beet.BeetArgsStruct<MintInstructionArgs & {
    instructionDiscriminator: number[];
}>;
export declare type MintInstructionAccounts = {
    candyMachine: web3.PublicKey;
    candyMachineCreator: web3.PublicKey;
    authority: web3.PublicKey;
    updateAuthority: web3.PublicKey;
    payer: web3.PublicKey;
    metadata: web3.PublicKey;
    mint: web3.PublicKey;
    mintAuthority: web3.PublicKey;
    mintUpdateAuthority: web3.PublicKey;
    masterEdition: web3.PublicKey;
    tokenMetadataProgram: web3.PublicKey;
    tokenProgram?: web3.PublicKey;
    systemProgram?: web3.PublicKey;
    rent?: web3.PublicKey;
    recentSlothashes: web3.PublicKey;
};
export declare const mintInstructionDiscriminator: number[];
export declare function createMintInstruction(accounts: MintInstructionAccounts, args: MintInstructionArgs, programId?: web3.PublicKey): web3.TransactionInstruction;
