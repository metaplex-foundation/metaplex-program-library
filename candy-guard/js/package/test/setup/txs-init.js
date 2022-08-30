"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.InitTransactions = void 0;
const amman_client_1 = require("@metaplex-foundation/amman-client");
const web3_js_1 = require("@solana/web3.js");
const _1 = require(".");
const utils_1 = require("../utils");
const generated_1 = require("../../src/generated");
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
    async initialize(t, data, payer, handler) {
        const [_, keypair] = await this.getKeypair('Candy Guard Base Pubkey');
        const pda = await (0, utils_1.getCandyGuardPDA)(generated_1.PROGRAM_ID, keypair);
        _1.amman.addr.addLabel('Candy Guard Account', pda);
        const accounts = {
            candyGuard: pda,
            base: keypair.publicKey,
            authority: payer.publicKey,
            payer: payer.publicKey,
            systemProgram: web3_js_1.SystemProgram.programId
        };
        const args = {
            data: data,
        };
        const tx = new web3_js_1.Transaction().add((0, generated_1.createInitializeInstruction)(accounts, args));
        return {
            tx: handler.sendAndConfirmTransaction(tx, [payer, keypair], 'tx: Initialize'),
            candyGuard: pda
        };
    }
    async update(t, candyGuard, data, payer, handler) {
        const accounts = {
            candyGuard,
            authority: payer.publicKey,
            payer: payer.publicKey,
            systemProgram: web3_js_1.SystemProgram.programId
        };
        const args = {
            data,
        };
        const tx = new web3_js_1.Transaction().add((0, generated_1.createUpdateInstruction)(accounts, args));
        return {
            tx: handler.sendAndConfirmTransaction(tx, [payer], 'tx: Update')
        };
    }
}
exports.InitTransactions = InitTransactions;
//# sourceMappingURL=txs-init.js.map