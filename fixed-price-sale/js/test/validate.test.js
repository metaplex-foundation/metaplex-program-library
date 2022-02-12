"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
var bn_js_1 = require("bn.js");
var tape_1 = require("tape");
var amman_1 = require("@metaplex-foundation/amman");
var mpl_token_metadata_1 = require("@metaplex-foundation/mpl-token-metadata");
var mpl_core_1 = require("@metaplex-foundation/mpl-core");
var utils_1 = require("../src/utils");
var transactions_1 = require("./transactions");
var utils_2 = require("./utils");
var actions_1 = require("./actions");
(0, utils_2.killStuckProcess)();
(0, tape_1)('validate: successful purchase and validation', function (t) { return __awaiter(void 0, void 0, void 0, function () {
    var _a, payer, connection, transactionHandler, store, _b, sellingResource, vault, vaultOwner, vaultOwnerBump, resourceMint, _c, treasuryMint, userTokenAcc, startDate, params, _d, market, treasuryHolder, _e, tradeHistory, tradeHistoryBump, _f, newMint, newMintAta, newMintEdition, newMintMetadata, resourceMintMasterEdition, resourceMintMetadata, resourceMintEditionMarker, buyTx, buyRes, me, ta, result;
    return __generator(this, function (_g) {
        switch (_g.label) {
            case 0: return [4 /*yield*/, (0, actions_1.createPrerequisites)()];
            case 1:
                _a = _g.sent(), payer = _a.payer, connection = _a.connection, transactionHandler = _a.transactionHandler;
                console.log("🚀 ~ file: validate.test.ts ~ line 28 ~ test ~ transactionHandler -  payer", transactionHandler["payer"].publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 28 ~ test ~ connection - rpcendpoint", connection["_rpcEndpoint"]);
                console.log("🚀 ~ file: validate.test.ts ~ line 28 ~ test ~ payer", payer.publicKey.toBase58());
                return [4 /*yield*/, (0, actions_1.createStore)({
                        test: t,
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                        params: {
                            name: 'Store',
                            description: 'Description',
                        },
                    })];
            case 2:
                store = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 42 ~ test ~ store", store.publicKey.toBase58());
                return [4 /*yield*/, (0, actions_1.initSellingResource)({
                        test: t,
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                        store: store.publicKey,
                        maxSupply: 100,
                    })];
            case 3:
                _b = _g.sent(), sellingResource = _b.sellingResource, vault = _b.vault, vaultOwner = _b.vaultOwner, vaultOwnerBump = _b.vaultOwnerBump, resourceMint = _b.resourceMint;
                console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ resourceMint", resourceMint.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ vaultOwnerBump", vaultOwnerBump);
                console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ vaultOwner", vaultOwner.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ vault", vault.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ sellingResource", sellingResource.publicKey.toBase58());
                return [4 /*yield*/, (0, actions_1.mintNFT)({
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                    })];
            case 4:
                _c = _g.sent(), treasuryMint = _c.mint, userTokenAcc = _c.tokenAccount;
                console.log("🚀 ~ file: validate.test.ts ~ line 60 ~ test ~ treasuryMint", treasuryMint.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 65 ~ test ~ userTokenAcc", userTokenAcc.publicKey.toBase58());
                startDate = Math.round(Date.now() / 1000);
                params = {
                    name: 'Market',
                    description: '',
                    startDate: startDate,
                    endDate: startDate + 5 * 20,
                    mutable: true,
                    price: 0.001,
                    piecesInOneWallet: 1,
                };
                return [4 /*yield*/, (0, actions_1.createMarket)({
                        test: t,
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                        store: store.publicKey,
                        sellingResource: sellingResource.publicKey,
                        treasuryMint: treasuryMint.publicKey,
                        params: params,
                    })];
            case 5:
                _d = _g.sent(), market = _d.market, treasuryHolder = _d.treasuryHolder;
                console.log("🚀 ~ file: validate.test.ts ~ line 79 ~ test ~ treasuryHolder", treasuryHolder.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 89 ~ test ~ market", market.publicKey.toBase58());
                return [4 /*yield*/, (0, utils_1.findTradeHistoryAddress)(payer.publicKey, market.publicKey)];
            case 6:
                _e = _g.sent(), tradeHistory = _e[0], tradeHistoryBump = _e[1];
                console.log("🚀 ~ file: validate.test.ts ~ line 92 ~ test ~ tradeHistoryBump", tradeHistoryBump);
                console.log("🚀 ~ file: validate.test.ts ~ line 95 ~ test ~ tradeHistory", tradeHistory.toBase58());
                return [4 /*yield*/, (0, actions_1.mintTokenToAccount)({
                        connection: connection,
                        payer: payer.publicKey,
                        transactionHandler: transactionHandler,
                    })];
            case 7:
                _f = _g.sent(), newMint = _f.mint, newMintAta = _f.mintAta;
                console.log("🚀 ~ file: validate.test.ts ~ line 99 ~ test ~ newMint", newMint.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 104 ~ test ~ newMintAta", newMintAta.publicKey.toBase58());
                (0, utils_2.logDebug)('new mint', newMint.publicKey.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Edition.getPDA(newMint.publicKey)];
            case 8:
                newMintEdition = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 109 ~ test ~ newMintEdition", newMintEdition.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Metadata.getPDA(newMint.publicKey)];
            case 9:
                newMintMetadata = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 111 ~ test ~ newMintMetadata", newMintMetadata.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Edition.getPDA(resourceMint.publicKey)];
            case 10:
                resourceMintMasterEdition = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 114 ~ test ~ resourceMintMasterEdition", resourceMintMasterEdition.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Metadata.getPDA(resourceMint.publicKey)];
            case 11:
                resourceMintMetadata = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 116 ~ test ~ resourceMintMetadata", resourceMintMetadata.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.EditionMarker.getPDA(resourceMint.publicKey, new bn_js_1(1))];
            case 12:
                resourceMintEditionMarker = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 118 ~ test ~ resourceMintEditionMarker", resourceMintEditionMarker.toBase58());
                return [4 /*yield*/, (0, utils_2.sleep)(1000)];
            case 13:
                _g.sent();
                return [4 /*yield*/, (0, transactions_1.createBuyTransaction)({
                        connection: connection,
                        buyer: payer.publicKey,
                        userTokenAccount: userTokenAcc.publicKey,
                        resourceMintMetadata: resourceMintMetadata,
                        resourceMintEditionMarker: resourceMintEditionMarker,
                        resourceMintMasterEdition: resourceMintMasterEdition,
                        sellingResource: sellingResource.publicKey,
                        market: market.publicKey,
                        marketTreasuryHolder: treasuryHolder.publicKey,
                        vaultOwner: vaultOwner,
                        tradeHistory: tradeHistory,
                        tradeHistoryBump: tradeHistoryBump,
                        vault: vault.publicKey,
                        vaultOwnerBump: vaultOwnerBump,
                        newMint: newMint.publicKey,
                        newMintEdition: newMintEdition,
                        newMintMetadata: newMintMetadata,
                    })];
            case 14:
                buyTx = (_g.sent()).tx;
                console.log("🚀 ~ file: validate.test.ts ~ line 123 ~ test ~ buyTx", buyTx);
                return [4 /*yield*/, transactionHandler.sendAndConfirmTransaction(buyTx, [payer], amman_1.defaultSendOptions)];
            case 15:
                buyRes = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 148 ~ test ~ buyRes", buyRes["txSignature"]);
                (0, utils_2.logDebug)('validate: successful purchase');
                (0, amman_1.assertConfirmedTransaction)(t, buyRes.txConfirmed);
                return [4 /*yield*/, mpl_token_metadata_1.MasterEdition.load(connection, resourceMintMasterEdition)];
            case 16:
                me = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 154 ~ test ~ MasterEdition", me.pubkey.toBase58());
                console.log("Built in console: ", "Master Edition me: ", me.pubkey.toString(), "resourceMintMasterEdition: ", resourceMintMasterEdition.toString(), "userTokenAcc: ", userTokenAcc.publicKey.toString());
                return [4 /*yield*/, mpl_core_1.TokenAccount.load(connection, newMintAta.publicKey)];
            case 17:
                ta = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 161 ~ test ~ TokenAccount", ta.pubkey.toBase58());
                return [4 /*yield*/, (0, utils_1.validateMembershipToken)(connection, me, ta)];
            case 18:
                result = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 163 ~ test ~ result", result);
                (0, utils_2.logDebug)('validate: copy is valid');
                t.equal(result, true);
                return [2 /*return*/];
        }
    });
}); });
(0, tape_1)('validate: successful purchase and failed validation', function (t) { return __awaiter(void 0, void 0, void 0, function () {
    var _a, payer, connection, transactionHandler, store, _b, sellingResource, vault, vaultOwner, vaultOwnerBump, resourceMint, _c, treasuryMint, userTokenAcc, startDate, params, _d, market, treasuryHolder, _e, tradeHistory, tradeHistoryBump, _f, newMint, newMintAta, newMintEdition, newMintMetadata, resourceMintMasterEdition, resourceMintMetadata, resourceMintEditionMarker, buyTx, buyRes, masterEdition, me, ta, result;
    return __generator(this, function (_g) {
        switch (_g.label) {
            case 0: return [4 /*yield*/, (0, actions_1.createPrerequisites)()];
            case 1:
                _a = _g.sent(), payer = _a.payer, connection = _a.connection, transactionHandler = _a.transactionHandler;
                return [4 /*yield*/, (0, actions_1.createStore)({
                        test: t,
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                        params: {
                            name: 'Store',
                            description: 'Description',
                        },
                    })];
            case 2:
                store = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 183 ~ test2 ~ store", store.publicKey.toBase58());
                return [4 /*yield*/, (0, actions_1.initSellingResource)({
                        test: t,
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                        store: store.publicKey,
                        maxSupply: 100,
                    })];
            case 3:
                _b = _g.sent(), sellingResource = _b.sellingResource, vault = _b.vault, vaultOwner = _b.vaultOwner, vaultOwnerBump = _b.vaultOwnerBump, resourceMint = _b.resourceMint;
                console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ resourceMint", resourceMint.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ vaultOwnerBump", vaultOwnerBump);
                console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ vaultOwner", vaultOwner.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ vault", vault.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ sellingResource", sellingResource.publicKey.toBase58());
                return [4 /*yield*/, (0, actions_1.mintNFT)({
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                    })];
            case 4:
                _c = _g.sent(), treasuryMint = _c.mint, userTokenAcc = _c.tokenAccount;
                console.log("🚀 ~ file: validate.test.ts ~ line 201 ~ test2 ~ treasuryMint", treasuryMint.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 206 ~ test2 ~ userTokenAcc", userTokenAcc.publicKey.toBase58());
                startDate = Math.round(Date.now() / 1000);
                params = {
                    name: 'Market',
                    description: '',
                    startDate: startDate,
                    endDate: startDate + 5 * 20,
                    mutable: true,
                    price: 0.001,
                    piecesInOneWallet: 1,
                };
                return [4 /*yield*/, (0, actions_1.createMarket)({
                        test: t,
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                        store: store.publicKey,
                        sellingResource: sellingResource.publicKey,
                        treasuryMint: treasuryMint.publicKey,
                        params: params,
                    })];
            case 5:
                _d = _g.sent(), market = _d.market, treasuryHolder = _d.treasuryHolder;
                console.log("🚀 ~ file: validate.test.ts ~ line 220 ~ test2 ~ market", market.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 220 ~ test2 ~ treasuryHolder", treasuryHolder.publicKey.toBase58());
                return [4 /*yield*/, (0, utils_1.findTradeHistoryAddress)(payer.publicKey, market.publicKey)];
            case 6:
                _e = _g.sent(), tradeHistory = _e[0], tradeHistoryBump = _e[1];
                console.log("🚀 ~ file: validate.test.ts ~ line 237 ~ test2 ~ tradeHistory", tradeHistory.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 233 ~ test2 ~ tradeHistoryBump", tradeHistoryBump);
                return [4 /*yield*/, (0, actions_1.mintTokenToAccount)({
                        connection: connection,
                        payer: payer.publicKey,
                        transactionHandler: transactionHandler,
                    })];
            case 7:
                _f = _g.sent(), newMint = _f.mint, newMintAta = _f.mintAta;
                console.log("🚀 ~ file: validate.test.ts ~ line 241 ~ test2 ~ newMint", newMint.publicKey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 246 ~ test2 ~ newMintAta", newMintAta.publicKey.toBase58());
                (0, utils_2.logDebug)('new mint', newMint.publicKey.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Edition.getPDA(newMint.publicKey)];
            case 8:
                newMintEdition = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 251 ~ test2 ~ newMintEdition", newMintEdition.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Metadata.getPDA(newMint.publicKey)];
            case 9:
                newMintMetadata = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 253 ~ test2 ~ newMintMetadata", newMintMetadata.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Edition.getPDA(resourceMint.publicKey)];
            case 10:
                resourceMintMasterEdition = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 256 ~ test2 ~ resourceMintMasterEdition", resourceMintMasterEdition.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Metadata.getPDA(resourceMint.publicKey)];
            case 11:
                resourceMintMetadata = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 257 ~ test2 ~ resourceMintMetadata", resourceMintMetadata.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.EditionMarker.getPDA(resourceMint.publicKey, new bn_js_1(1))];
            case 12:
                resourceMintEditionMarker = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 260 ~ test2 ~ resourceMintEditionMarker", resourceMintEditionMarker.toBase58());
                return [4 /*yield*/, (0, utils_2.sleep)(1000)];
            case 13:
                _g.sent();
                return [4 /*yield*/, (0, transactions_1.createBuyTransaction)({
                        connection: connection,
                        buyer: payer.publicKey,
                        userTokenAccount: userTokenAcc.publicKey,
                        resourceMintMetadata: resourceMintMetadata,
                        resourceMintEditionMarker: resourceMintEditionMarker,
                        resourceMintMasterEdition: resourceMintMasterEdition,
                        sellingResource: sellingResource.publicKey,
                        market: market.publicKey,
                        marketTreasuryHolder: treasuryHolder.publicKey,
                        vaultOwner: vaultOwner,
                        tradeHistory: tradeHistory,
                        tradeHistoryBump: tradeHistoryBump,
                        vault: vault.publicKey,
                        vaultOwnerBump: vaultOwnerBump,
                        newMint: newMint.publicKey,
                        newMintEdition: newMintEdition,
                        newMintMetadata: newMintMetadata,
                    })];
            case 14:
                buyTx = (_g.sent()).tx;
                console.log("🚀 ~ file: validate.test.ts ~ line 265 ~ test2 ~ buyTx", buyTx);
                return [4 /*yield*/, transactionHandler.sendAndConfirmTransaction(buyTx, [payer], amman_1.defaultSendOptions)];
            case 15:
                buyRes = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 291 ~ test2 ~ buyRes", buyRes.txSignature);
                (0, utils_2.logDebug)('validate: successful purchase');
                (0, amman_1.assertConfirmedTransaction)(t, buyRes.txConfirmed);
                return [4 /*yield*/, (0, actions_1.mintNFT)({
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                    })];
            case 16:
                masterEdition = (_g.sent()).edition;
                console.log("🚀 ~ file: validate.test.ts ~ line 297 ~ test2 ~ masterEdition", masterEdition.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.MasterEdition.load(connection, masterEdition)];
            case 17:
                me = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 280 ~ test2 ~ me", me.pubkey.toBase58());
                console.log("🚀 ~ file: validate.test.ts ~ line 280 ~ test2 ~ masterEdition", masterEdition.toBase58());
                return [4 /*yield*/, mpl_core_1.TokenAccount.load(connection, newMintAta.publicKey)];
            case 18:
                ta = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 308 ~ test2 ~ ta", ta.pubkey.toBase58());
                return [4 /*yield*/, (0, utils_1.validateMembershipToken)(connection, me, ta)];
            case 19:
                result = _g.sent();
                console.log("🚀 ~ file: validate.test.ts ~ line 310 ~ test2 ~ result", result);
                (0, utils_2.logDebug)('validate: copy is invalid');
                t.equal(result, false);
                return [2 /*return*/];
        }
    });
}); });
