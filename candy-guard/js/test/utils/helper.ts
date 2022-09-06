import { Test } from 'tape'
import {
    ConfirmedTransactionAssertablePromise,
    PayerTransactionHandler
} from '@metaplex-foundation/amman-client'
import {
    Connection,
    Keypair,
    PublicKey,
    SystemProgram,
    Transaction,
    TransactionInstruction,
    SYSVAR_RENT_PUBKEY,
    SYSVAR_SLOT_HASHES_PUBKEY,
} from '@solana/web3.js'
import {
    AddConfigLinesInstructionAccounts,
    AddConfigLinesInstructionArgs,
    CandyMachine,
    CandyMachineData,
    ConfigLine,
    createAddConfigLinesInstruction,
    createInitializeInstruction,
    createMintInstruction,
    InitializeInstructionAccounts,
    InitializeInstructionArgs,
    MintInstructionAccounts,
    MintInstructionArgs,
    PROGRAM_ID
} from '../../../../candy-core/js/src/generated'
import { getCandyMachineSpace } from '../../../../candy-core/js/test/utils'
import { amman } from '../setup'
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    createAssociatedTokenAccountInstruction,
    createInitializeMintInstruction,
    createMintToInstruction,
    MintLayout,
    TOKEN_PROGRAM_ID
} from '@solana/spl-token'

export const CANDY_MACHINE_PROGRAM = PROGRAM_ID;
export const METAPLEX_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

export class CandyMachineHelper {
    async create(
        t: Test,
        payer: Keypair,
        address: Keypair,
        data: CandyMachineData,
        handler: PayerTransactionHandler,
        connection: Connection
    ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {

        const accounts: InitializeInstructionAccounts = {
            candyMachine: address.publicKey,
            wallet: payer.publicKey,
            authority: payer.publicKey,
            updateAuthority: payer.publicKey,
            payer: payer.publicKey,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
        };

        const args: InitializeInstructionArgs = {
            data: data,
        };

        const ixInitialize = createInitializeInstruction(accounts, args);
        const ixCreateAccount = SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: address.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(
                getCandyMachineSpace(data),
            ),
            space: getCandyMachineSpace(data),
            programId: PROGRAM_ID,
        });

        const tx = new Transaction().add(ixCreateAccount).add(ixInitialize);

        return {
            tx: handler
                .sendAndConfirmTransaction(tx, [address, payer], 'tx: Initialize')
        };
    }

    async addConfigLines(
        t: Test,
        candyMachine: PublicKey,
        payer: Keypair,
        lines: ConfigLine[],
        handler: PayerTransactionHandler
    ): Promise<{ txs: Transaction[] }> {
        const accounts: AddConfigLinesInstructionAccounts = {
            candyMachine: candyMachine,
            authority: payer.publicKey,
        };

        const txs: Transaction[] = [];
        let start = 0;

        while (start < lines.length) {
            // sends the config lines in chunks of 10
            const limit = Math.min(lines.length - start, 10);
            const args: AddConfigLinesInstructionArgs = {
                configLines: lines.slice(start, start + limit),
                index: start
            };

            const ix = createAddConfigLinesInstruction(accounts, args);
            txs.push(new Transaction().add(ix));

            start = start + limit;
        }

        return { txs };
    }

    async mint(
        t: Test,
        candyMachine: PublicKey,
        payer: Keypair,
        mint: Keypair,
        handler: PayerTransactionHandler,
        connection: Connection
    ): Promise<{ tx: ConfirmedTransactionAssertablePromise }> {
        const candyMachineObject = await CandyMachine.fromAccountAddress(connection, candyMachine);
        
        // PDAs required for the mint

        // creator address
        const [candyMachineCreator, bump] = await PublicKey.findProgramAddress(
            [Buffer.from('candy_machine'), candyMachine.toBuffer()],
            PROGRAM_ID,
        );
        amman.addr.addLabel('Mint Creator', candyMachineCreator);

        // associated token address
        const [associatedToken,] = await PublicKey.findProgramAddress(
            [payer.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
            ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        amman.addr.addLabel('Mint Associated Token', associatedToken);

        // metadata address
        const [metadataAddress,] = await PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
            ],
            METAPLEX_PROGRAM_ID,
        );
        amman.addr.addLabel('Mint Metadata', metadataAddress);

        // master edition address
        const [masterEdition,] = await PublicKey.findProgramAddress(
            [
                Buffer.from('metadata'),
                METAPLEX_PROGRAM_ID.toBuffer(),
                mint.publicKey.toBuffer(),
                Buffer.from('edition'),
            ],
            METAPLEX_PROGRAM_ID,
        );
        amman.addr.addLabel('Mint Master Edition', masterEdition);

        const accounts: MintInstructionAccounts = {
            candyMachine: candyMachine,
            authority: candyMachineObject.authority,
            updateAuthority: candyMachineObject.updateAuthority,
            candyMachineCreator: candyMachineCreator,
            masterEdition: masterEdition,
            metadata: metadataAddress,
            mint: mint.publicKey,
            mintAuthority: payer.publicKey,
            mintUpdateAuthority: payer.publicKey,
            payer: payer.publicKey,
            tokenMetadataProgram: METAPLEX_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
            recentSlothashes: SYSVAR_SLOT_HASHES_PUBKEY
        };

        const args: MintInstructionArgs = {
            creatorBump: bump
        };

        const ixs: TransactionInstruction[] = [];
        ixs.push(SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: mint.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(
                MintLayout.span,
            ),
            space: MintLayout.span,
            programId: TOKEN_PROGRAM_ID,
        }));
        ixs.push(createInitializeMintInstruction(mint.publicKey, 0, payer.publicKey, payer.publicKey));
        ixs.push(createAssociatedTokenAccountInstruction(payer.publicKey, associatedToken, payer.publicKey, mint.publicKey));
        ixs.push(createMintToInstruction(mint.publicKey, associatedToken, payer.publicKey, 1, []));
        // candy machine mint instruction
        ixs.push(createMintInstruction(accounts, args));
        const tx = new Transaction().add(...ixs);

        return { tx: handler.sendAndConfirmTransaction(tx, [payer, mint], 'tx: Candy Machine Mint') };
    }
}