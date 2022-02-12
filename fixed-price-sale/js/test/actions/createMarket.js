"use strict";
var __assign = (this && this.__assign) || function () {
    __assign = Object.assign || function(t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
            s = arguments[i];
            for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
                t[p] = s[p];
        }
        return t;
    };
    return __assign.apply(this, arguments);
};
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
exports.createMarket = void 0;
var amman_1 = require("@metaplex-foundation/amman");
var web3_js_1 = require("@solana/web3.js");
var transactions_1 = require("../transactions");
var utils_1 = require("../utils");
var utils_2 = require("../../src/utils");
var instructions_1 = require("../../src/instructions");
var createMarket = function (_a) {
    var test = _a.test, transactionHandler = _a.transactionHandler, payer = _a.payer, connection = _a.connection, store = _a.store, sellingResource = _a.sellingResource, treasuryMint = _a.treasuryMint, params = _a.params;
    return __awaiter(void 0, void 0, void 0, function () {
        var _b, treasuryOwner, treasuryOwnerBump, _c, treasuryHolder, createTokenTx, createVaultRes, market, instruction, marketTx, marketRes;
        return __generator(this, function (_d) {
            switch (_d.label) {
                case 0: return [4 /*yield*/, (0, utils_2.findTresuryOwnerAddress)(treasuryMint, sellingResource)];
                case 1:
                    _b = _d.sent(), treasuryOwner = _b[0], treasuryOwnerBump = _b[1];
                    (0, utils_1.logDebug)("treasuryOwner: ".concat(treasuryOwner.toBase58()));
                    return [4 /*yield*/, (0, transactions_1.createTokenAccount)({
                            payer: payer.publicKey,
                            connection: connection,
                            mint: treasuryMint,
                            owner: treasuryOwner,
                        })];
                case 2:
                    _c = _d.sent(), treasuryHolder = _c.tokenAccount, createTokenTx = _c.createTokenTx;
                    return [4 /*yield*/, transactionHandler.sendAndConfirmTransaction(createTokenTx, [treasuryHolder], amman_1.defaultSendOptions)];
                case 3:
                    createVaultRes = _d.sent();
                    (0, utils_1.logDebug)("treasuryHolder: ".concat(treasuryHolder.publicKey));
                    (0, amman_1.assertConfirmedTransaction)(test, createVaultRes.txConfirmed);
                    market = web3_js_1.Keypair.generate();
                    instruction = (0, instructions_1.createCreateMarketInstruction)({
                        market: market.publicKey,
                        store: store,
                        sellingResourceOwner: payer.publicKey,
                        sellingResource: sellingResource,
                        mint: treasuryMint,
                        treasuryHolder: treasuryHolder.publicKey,
                        owner: treasuryOwner,
                    }, __assign({ treasuryOwnerBump: treasuryOwnerBump }, params));
                    return [4 /*yield*/, (0, utils_1.createAndSignTransaction)(connection, payer, [instruction], [market])];
                case 4:
                    marketTx = _d.sent();
                    return [4 /*yield*/, transactionHandler.sendAndConfirmTransaction(marketTx, [market], amman_1.defaultSendOptions)];
                case 5:
                    marketRes = _d.sent();
                    (0, utils_1.logDebug)("market: ".concat(market.publicKey));
                    (0, amman_1.assertConfirmedTransaction)(test, marketRes.txConfirmed);
                    return [2 /*return*/, { market: market, treasuryHolder: treasuryHolder }];
            }
        });
    });
};
exports.createMarket = createMarket;
