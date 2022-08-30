"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.InitTransactions = void 0;
const amman_client_1 = require("@metaplex-foundation/amman-client");
const web3_js_1 = require("@solana/web3.js");
const spl_token_1 = require("@solana/spl-token");
const program = __importStar(require("../../src/generated"));
const _1 = require(".");
const utils_1 = require("../utils");
const generated_1 = require("../../src/generated");
const METAPLEX_PROGRAM_ID = new web3_js_1.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
class InitTransactions {
    constructor(resuseKeypairs = false) {
        this.resuseKeypairs = resuseKeypairs;
        this.getKeypair = resuseKeypairs
            ? _1.amman.loadOrGenKeypair
            : _1.amman.genLabeledKeypair;
    }
    async payer() {
        const [payer, payerPair] = await this.getKeypair('Payer');
        const connection = new web3_js_1.Connection(amman_client_1.LOCALHOST, 'confirmed');
        await _1.amman.airdrop(connection, payer, 2);
        const transactionHandler = _1.amman.payerTransactionHandler(connection, payerPair);
        return {
            fstTxHandler: transactionHandler,
            connection,
            payer,
            payerPair,
        };
    }
    async authority() {
        const [authority, authorityPair] = await this.getKeypair('Authority');
        const connection = new web3_js_1.Connection(amman_client_1.LOCALHOST, 'confirmed');
        await _1.amman.airdrop(connection, authority, 2);
        const transactionHandler = _1.amman.payerTransactionHandler(connection, authorityPair);
        return {
            fstTxHandler: transactionHandler,
            connection,
            authority,
            authorityPair,
        };
    }
    async minter() {
        const [minter, minterPair] = await this.getKeypair('Minter');
        const connection = new web3_js_1.Connection(amman_client_1.LOCALHOST, 'confirmed');
        await _1.amman.airdrop(connection, minter, 2);
        const transactionHandler = _1.amman.payerTransactionHandler(connection, minterPair);
        return {
            fstTxHandler: transactionHandler,
            connection,
            minter,
            minterPair,
        };
    }
    async create(t, payer, data, handler, connection) {
        const [_, candyMachine] = await this.getKeypair('Candy Machine Account');
        const accounts = {
            candyMachine: candyMachine.publicKey,
            wallet: payer.publicKey,
            authority: payer.publicKey,
            updateAuthority: payer.publicKey,
            payer: payer.publicKey,
            systemProgram: web3_js_1.SystemProgram.programId,
            rent: web3_js_1.SYSVAR_RENT_PUBKEY,
        };
        const args = {
            data: data,
        };
        const ixInitialize = program.createInitializeInstruction(accounts, args);
        const ixCreateAccount = web3_js_1.SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: candyMachine.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption((0, utils_1.getCandyMachineSpace)(data)),
            space: (0, utils_1.getCandyMachineSpace)(data),
            programId: program.PROGRAM_ID,
        });
        const tx = new web3_js_1.Transaction().add(ixCreateAccount).add(ixInitialize);
        const txPromise = handler
            .sendAndConfirmTransaction(tx, [candyMachine, payer], 'tx: Initialize');
        return { tx: txPromise, candyMachine: candyMachine.publicKey };
    }
    async addConfigLines(t, candyMachine, payer, lines, handler) {
        const accounts = {
            candyMachine: candyMachine,
            authority: payer.publicKey,
        };
        const txs = [];
        let start = 0;
        while (start < lines.length) {
            const limit = Math.min(lines.length - start, 10);
            const args = {
                configLines: lines.slice(start, start + limit),
                index: start
            };
            const ix = program.createAddConfigLinesInstruction(accounts, args);
            txs.push(new web3_js_1.Transaction().add(ix));
            start = start + limit;
        }
        return { txs };
    }
    async updateCandyMachine(t, candyMachine, wallet, payer, data, handler) {
        const accounts = {
            candyMachine: candyMachine,
            authority: payer.publicKey,
            wallet: wallet
        };
        const args = {
            data: data
        };
        const ix = program.createUpdateInstruction(accounts, args);
        const tx = new web3_js_1.Transaction().add(ix);
        return { tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Update') };
    }
    async mint(t, candyMachine, payer, handler, connection) {
        const candyMachineObject = await generated_1.CandyMachine.fromAccountAddress(connection, candyMachine);
        const [mint, mintPair] = await this.getKeypair('mint');
        _1.amman.addr.addLabel('Mint', mint);
        const [candyMachineCreator, bump] = await web3_js_1.PublicKey.findProgramAddress([Buffer.from('candy_machine'), candyMachine.toBuffer()], program.PROGRAM_ID);
        _1.amman.addr.addLabel('Mint Creator', candyMachineCreator);
        const [associatedToken,] = await web3_js_1.PublicKey.findProgramAddress([payer.publicKey.toBuffer(), spl_token_1.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()], spl_token_1.ASSOCIATED_TOKEN_PROGRAM_ID);
        _1.amman.addr.addLabel('Mint Associated Token', associatedToken);
        const [metadataAddress,] = await web3_js_1.PublicKey.findProgramAddress([
            Buffer.from('metadata'),
            METAPLEX_PROGRAM_ID.toBuffer(),
            mint.toBuffer(),
        ], METAPLEX_PROGRAM_ID);
        _1.amman.addr.addLabel('Mint Metadata', metadataAddress);
        const [masterEdition,] = await web3_js_1.PublicKey.findProgramAddress([
            Buffer.from('metadata'),
            METAPLEX_PROGRAM_ID.toBuffer(),
            mint.toBuffer(),
            Buffer.from('edition'),
        ], METAPLEX_PROGRAM_ID);
        _1.amman.addr.addLabel('Mint Master Edition', masterEdition);
        const accounts = {
            candyMachine: candyMachine,
            authority: candyMachineObject.authority,
            updateAuthority: candyMachineObject.updateAuthority,
            candyMachineCreator: candyMachineCreator,
            masterEdition: masterEdition,
            metadata: metadataAddress,
            mint: mint,
            mintAuthority: payer.publicKey,
            mintUpdateAuthority: payer.publicKey,
            payer: payer.publicKey,
            tokenMetadataProgram: METAPLEX_PROGRAM_ID,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
            systemProgram: web3_js_1.SystemProgram.programId,
            rent: web3_js_1.SYSVAR_RENT_PUBKEY,
            recentSlothashes: web3_js_1.SYSVAR_SLOT_HASHES_PUBKEY
        };
        const args = {
            creatorBump: bump
        };
        const ixs = [];
        ixs.push(web3_js_1.SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: mint,
            lamports: await connection.getMinimumBalanceForRentExemption(spl_token_1.MintLayout.span),
            space: spl_token_1.MintLayout.span,
            programId: spl_token_1.TOKEN_PROGRAM_ID,
        }));
        ixs.push((0, spl_token_1.createInitializeMintInstruction)(mint, 0, payer.publicKey, payer.publicKey));
        ixs.push((0, spl_token_1.createAssociatedTokenAccountInstruction)(payer.publicKey, associatedToken, payer.publicKey, mint));
        ixs.push((0, spl_token_1.createMintToInstruction)(mint, associatedToken, payer.publicKey, 1, []));
        ixs.push(program.createMintInstruction(accounts, args));
        const tx = new web3_js_1.Transaction().add(...ixs);
        return { tx: handler.sendAndConfirmTransaction(tx, [payer, mintPair], 'tx: Mint') };
    }
    async withdraw(t, candyMachine, payer, handler) {
        const accounts = {
            candyMachine: candyMachine,
            authority: payer.publicKey
        };
        const ix = program.createWithdrawInstruction(accounts);
        const tx = new web3_js_1.Transaction().add(ix);
        return { tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Withdraw') };
    }
}
exports.InitTransactions = InitTransactions;
//# sourceMappingURL=txs-init.js.map