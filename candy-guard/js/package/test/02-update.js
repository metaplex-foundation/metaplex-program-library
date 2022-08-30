"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const tape_1 = __importDefault(require("tape"));
const spok_1 = __importDefault(require("spok"));
const setup_1 = require("./setup");
const generated_1 = require("../src/generated");
const utils_1 = require("./utils");
const bn_js_1 = require("bn.js");
const API = new setup_1.InitTransactions();
(0, setup_1.killStuckProcess)();
(0, tape_1.default)('update: enable guards', async (t) => {
    const { fstTxHandler, payerPair, connection } = await API.payer();
    const data = {
        botTax: null,
        liveDate: null,
        lamports: null,
        splToken: null,
        thirdPartySigner: null,
        whitelist: null,
        gatekeeper: null,
        endSettings: null
    };
    const { tx: transaction, candyGuard: address } = await API.initialize(t, data, payerPair, fstTxHandler);
    await transaction.assertSuccess(t);
    let accountInfo = await connection.getAccountInfo(payerPair.publicKey);
    const balance = accountInfo === null || accountInfo === void 0 ? void 0 : accountInfo.lamports;
    const updateData = {
        botTax: {
            lamports: new bn_js_1.BN(100000000),
            lastInstruction: true
        },
        liveDate: {
            date: null
        },
        lamports: {
            amount: new bn_js_1.BN(100000000)
        },
        splToken: null,
        thirdPartySigner: null,
        whitelist: null,
        gatekeeper: null,
        endSettings: null
    };
    const { tx: updateTransaction } = await API.update(t, address, updateData, payerPair, fstTxHandler);
    await updateTransaction.assertSuccess(t);
    const candyGuard = await generated_1.CandyGuard.fromAccountAddress(connection, address);
    (0, spok_1.default)(t, candyGuard, {
        features: (0, utils_1.spokSameBignum)(7),
        authority: (0, utils_1.spokSamePubkey)(payerPair.publicKey),
    });
    accountInfo = await connection.getAccountInfo(payerPair.publicKey);
    const updatedBalance = accountInfo === null || accountInfo === void 0 ? void 0 : accountInfo.lamports;
    t.true(updatedBalance < balance, 'balance after update must be lower');
});
(0, tape_1.default)('update: disable guards', async (t) => {
    const { fstTxHandler, payerPair, connection } = await API.payer();
    const data = {
        botTax: {
            lamports: new bn_js_1.BN(100000000),
            lastInstruction: true
        },
        liveDate: {
            date: null
        },
        lamports: {
            amount: new bn_js_1.BN(100000000)
        },
        splToken: null,
        thirdPartySigner: null,
        whitelist: null,
        gatekeeper: null,
        endSettings: null
    };
    const { tx: transaction, candyGuard: address } = await API.initialize(t, data, payerPair, fstTxHandler);
    await transaction.assertSuccess(t);
    let accountInfo = await connection.getAccountInfo(payerPair.publicKey);
    const balance = accountInfo === null || accountInfo === void 0 ? void 0 : accountInfo.lamports;
    const updateData = {
        botTax: null,
        liveDate: null,
        lamports: null,
        splToken: null,
        thirdPartySigner: null,
        whitelist: null,
        gatekeeper: null,
        endSettings: null
    };
    const { tx: updateTransaction } = await API.update(t, address, updateData, payerPair, fstTxHandler);
    await updateTransaction.assertSuccess(t);
    const candyGuard = await generated_1.CandyGuard.fromAccountAddress(connection, address);
    (0, spok_1.default)(t, candyGuard, {
        features: (0, utils_1.spokSameBignum)(0),
        authority: (0, utils_1.spokSamePubkey)(payerPair.publicKey),
    });
    accountInfo = await connection.getAccountInfo(payerPair.publicKey);
    const updatedBalance = accountInfo === null || accountInfo === void 0 ? void 0 : accountInfo.lamports;
    t.true(updatedBalance > balance, 'balance after update must be greater');
});
//# sourceMappingURL=02-update.js.map