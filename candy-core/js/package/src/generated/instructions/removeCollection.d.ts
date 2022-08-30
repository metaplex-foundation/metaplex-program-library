import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
export declare const removeCollectionStruct: beet.BeetArgsStruct<{
    instructionDiscriminator: number[];
}>;
export declare type RemoveCollectionInstructionAccounts = {
    candyMachine: web3.PublicKey;
    authority: web3.PublicKey;
    updateAuthority: web3.PublicKey;
    collectionAuthority: web3.PublicKey;
    collectionMint: web3.PublicKey;
    collectionMetadata: web3.PublicKey;
    collectionAuthorityRecord: web3.PublicKey;
    tokenMetadataProgram: web3.PublicKey;
};
export declare const removeCollectionInstructionDiscriminator: number[];
export declare function createRemoveCollectionInstruction(accounts: RemoveCollectionInstructionAccounts, programId?: web3.PublicKey): web3.TransactionInstruction;
