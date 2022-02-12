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
var utils_1 = require("../src/utils");
var transactions_1 = require("./transactions");
var actions_1 = require("./actions");
var utils_2 = require("./utils");
(0, utils_2.killStuckProcess)();
(0, tape_1)('buy: successful purchase for newly minted treasury mint', function (t) { return __awaiter(void 0, void 0, void 0, function () {
    var _a, payer, connection, transactionHandler, store, _b, sellingResource, vault, vaultOwner, vaultOwnerBump, resourceMint, _c, treasuryMint, userTokenAcc, startDate, params, _d, market, treasuryHolder, _e, tradeHistory, tradeHistoryBump, newMint, newMintEdition, newMintMetadata, resourceMintMasterEdition, resourceMintMetadata, resourceMintEditionMarker, buyTx, buyRes;
    return __generator(this, function (_f) {
        switch (_f.label) {
            case 0: return [4 /*yield*/, (0, actions_1.createPrerequisites)()];
            case 1:
                _a = _f.sent(), payer = _a.payer, connection = _a.connection, transactionHandler = _a.transactionHandler;
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
                store = _f.sent();
                console.log("🚀 ~ file: buy.test.ts ~ line 33 ~ test ~ store", store.publicKey.toBase58());
                return [4 /*yield*/, (0, actions_1.initSellingResource)({
                        test: t,
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                        store: store.publicKey,
                        maxSupply: 100,
                    })];
            case 3:
                _b = _f.sent(), sellingResource = _b.sellingResource, vault = _b.vault, vaultOwner = _b.vaultOwner, vaultOwnerBump = _b.vaultOwnerBump, resourceMint = _b.resourceMint;
                console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ resourceMint", resourceMint.publicKey.toBase58());
                console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ vaultOwnerBump", vaultOwnerBump);
                console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ vaultOwner", vaultOwner.toBase58());
                console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ vault", vault.publicKey.toBase58());
                console.log("🚀 ~ file: buy.test.ts ~ line 36 ~ test ~ sellingResource", sellingResource.publicKey.toBase58());
                return [4 /*yield*/, (0, actions_1.mintNFT)({
                        transactionHandler: transactionHandler,
                        payer: payer,
                        connection: connection,
                    })];
            case 4:
                _c = _f.sent(), treasuryMint = _c.mint, userTokenAcc = _c.tokenAccount;
                console.log("🚀 ~ file: buy.test.ts ~ line 51 ~ test ~ treasuryMint", treasuryMint.publicKey.toBase58());
                console.log("🚀 ~ file: buy.test.ts ~ line 56 ~ test ~ userTokenAcc", userTokenAcc.publicKey.toBase58());
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
                _d = _f.sent(), market = _d.market, treasuryHolder = _d.treasuryHolder;
                console.log("🚀 ~ file: buy.test.ts ~ line 72 ~ test ~ treasuryHolder", treasuryHolder.publicKey.toBase58());
                console.log("🚀 ~ file: buy.test.ts ~ line 72 ~ test ~ market", market.publicKey.toBase58());
                return [4 /*yield*/, (0, utils_1.findTradeHistoryAddress)(payer.publicKey, market.publicKey)];
            case 6:
                _e = _f.sent(), tradeHistory = _e[0], tradeHistoryBump = _e[1];
                console.log("🚀 ~ file: buy.test.ts ~ line 85 ~ test ~ tradeHistoryBump", tradeHistoryBump);
                console.log("🚀 ~ file: buy.test.ts ~ line 88 ~ test ~ tradeHistory", tradeHistory.toBase58());
                return [4 /*yield*/, (0, actions_1.mintTokenToAccount)({
                        connection: connection,
                        payer: payer.publicKey,
                        transactionHandler: transactionHandler,
                    })];
            case 7:
                newMint = (_f.sent()).mint;
                console.log("🚀 ~ file: buy.test.ts ~ line 92 ~ test ~ newMint", newMint.publicKey.toBase58());
                (0, utils_2.logDebug)('new mint', newMint.publicKey.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Edition.getPDA(newMint.publicKey)];
            case 8:
                newMintEdition = _f.sent();
                console.log("🚀 ~ file: buy.test.ts ~ line 101 ~ test ~ newMintEdition", newMintEdition.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Metadata.getPDA(newMint.publicKey)];
            case 9:
                newMintMetadata = _f.sent();
                console.log("🚀 ~ file: buy.test.ts ~ line 103 ~ test ~ newMintMetadata", newMintMetadata.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Edition.getPDA(resourceMint.publicKey)];
            case 10:
                resourceMintMasterEdition = _f.sent();
                console.log("🚀 ~ file: buy.test.ts ~ line 106 ~ test ~ resourceMintMasterEdition", resourceMintMasterEdition.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.Metadata.getPDA(resourceMint.publicKey)];
            case 11:
                resourceMintMetadata = _f.sent();
                console.log("🚀 ~ file: buy.test.ts ~ line 108 ~ test ~ resourceMintMetadata", resourceMintMetadata.toBase58());
                return [4 /*yield*/, mpl_token_metadata_1.EditionMarker.getPDA(resourceMint.publicKey, new bn_js_1(1))];
            case 12:
                resourceMintEditionMarker = _f.sent();
                console.log("🚀 ~ file: buy.test.ts ~ line 109 ~ test ~ resourceMintEditionMarker", resourceMintEditionMarker.toBase58());
                return [4 /*yield*/, (0, utils_2.sleep)(1000)];
            case 13:
                _f.sent();
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
                buyTx = (_f.sent()).tx;
                console.log("🚀 ~ file: buy.test.ts ~ line 116 ~ test ~ buyTx", buyTx);
                return [4 /*yield*/, transactionHandler.sendAndConfirmTransaction(buyTx, [payer], amman_1.defaultSendOptions)];
            case 15:
                buyRes = _f.sent();
                console.log("🚀 ~ file: buy.test.ts ~ line 142 ~ test ~ buyRes", buyRes.txSignature);
                console.log("End to End complete for buy method");
                (0, utils_2.logDebug)('buy:: successful purchase');
                (0, amman_1.assertConfirmedTransaction)(t, buyRes.txConfirmed);
                return [2 /*return*/];
        }
    });
}); });
