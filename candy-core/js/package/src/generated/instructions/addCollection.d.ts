import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
export declare const addCollectionStruct: beet.BeetArgsStruct<{
    instructionDiscriminator: number[];
}>;
export declare type AddCollectionInstructionAccounts = {
    candyMachine: web3.PublicKey;
    authority: web3.PublicKey;
    updateAuthority: web3.PublicKey;
    payer: web3.PublicKey;
    collectionAuthority: web3.PublicKey;
    collectionMetadata: web3.PublicKey;
    collectionMint: web3.PublicKey;
    collectionEdition: web3.PublicKey;
    collectionAuthorityRecord: web3.PublicKey;
    tokenMetadataProgram: web3.PublicKey;
    systemProgram?: web3.PublicKey;
    rent?: web3.PublicKey;
};
export declare const addCollectionInstructionDiscriminator: number[];
export declare function createAddCollectionInstruction(accounts: AddCollectionInstructionAccounts, programId?: web3.PublicKey): web3.TransactionInstruction;
