"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const tape_1 = __importDefault(require("tape"));
const spok_1 = __importDefault(require("spok"));
const bn_js_1 = require("bn.js");
const setup_1 = require("./setup/");
const generated_1 = require("../src/generated");
const utils_1 = require("./utils");
const src_1 = require("../src");
const API = new setup_1.InitTransactions();
(0, setup_1.killStuckProcess)();
(0, tape_1.default)('initialize: new candy guard (no guards)', async (t) => {
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
    const candyGuard = await generated_1.CandyGuard.fromAccountAddress(connection, address);
    (0, spok_1.default)(t, candyGuard, {
        features: (0, utils_1.spokSameBignum)(0),
        authority: (0, utils_1.spokSamePubkey)(payerPair.publicKey),
    });
});
(0, tape_1.default)('initialize: new candy guard (with guards)', async (t) => {
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
    const candyGuard = await generated_1.CandyGuard.fromAccountAddress(connection, address);
    (0, spok_1.default)(t, candyGuard, {
        features: (0, utils_1.spokSameBignum)(7),
        authority: (0, utils_1.spokSamePubkey)(payerPair.publicKey),
    });
    let accountInfo = await connection.getAccountInfo(address);
    const candyGuardData = (0, src_1.parseData)(candyGuard, accountInfo === null || accountInfo === void 0 ? void 0 : accountInfo.data.subarray(utils_1.DATA_OFFSET));
    (0, spok_1.default)(t, candyGuardData.lamports, {
        amount: (0, utils_1.spokSameBignum)(data.lamports.amount)
    });
});
//# sourceMappingURL=01-initialize.js.map