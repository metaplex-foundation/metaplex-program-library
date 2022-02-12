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
exports.createBuyTransaction = void 0;
var web3_js_1 = require("@solana/web3.js");
var mpl_token_metadata_1 = require("@metaplex-foundation/mpl-token-metadata");
var instructions_1 = require("../../src/instructions");
var createBuyTransaction = function (_a) {
    var connection = _a.connection, buyer = _a.buyer, userTokenAccount = _a.userTokenAccount, resourceMintMetadata = _a.resourceMintMetadata, resourceMintEditionMarker = _a.resourceMintEditionMarker, resourceMintMasterEdition = _a.resourceMintMasterEdition, sellingResource = _a.sellingResource, tradeHistory = _a.tradeHistory, tradeHistoryBump = _a.tradeHistoryBump, market = _a.market, marketTreasuryHolder = _a.marketTreasuryHolder, vault = _a.vault, vaultOwner = _a.vaultOwner, vaultOwnerBump = _a.vaultOwnerBump, newMint = _a.newMint, newMintEdition = _a.newMintEdition, newMintMetadata = _a.newMintMetadata;
    return __awaiter(void 0, void 0, void 0, function () {
        var instruction, tx, _b;
        return __generator(this, function (_c) {
            switch (_c.label) {
                case 0: return [4 /*yield*/, (0, instructions_1.createBuyInstruction)({
                        // buyer wallet
                        userWallet: buyer,
                        // user token account
                        userTokenAccount: userTokenAccount,
                        // resource mint edition marker PDA
                        editionMarker: resourceMintEditionMarker,
                        // resource mint master edition
                        masterEdition: resourceMintMasterEdition,
                        // resource mint metadata PDA
                        masterEditionMetadata: resourceMintMetadata,
                        // token account for selling resource
                        vault: vault,
                        // account which holds selling entities
                        sellingResource: sellingResource,
                        // owner of selling resource token account PDA
                        owner: vaultOwner,
                        // market account
                        market: market,
                        // PDA which creates on market for each buyer
                        tradeHistory: tradeHistory,
                        // market treasury holder (buyer will send tokens to this account)
                        treasuryHolder: marketTreasuryHolder,
                        // newly generated mint address
                        newMint: newMint,
                        // newly generated mint metadata PDA
                        newMetadata: newMintMetadata,
                        // newly generated mint edition PDA
                        newEdition: newMintEdition,
                        // solana system account
                        clock: web3_js_1.SYSVAR_CLOCK_PUBKEY,
                        // metaplex token metadata program address
                        tokenMetadataProgram: mpl_token_metadata_1.MetadataProgram.PUBKEY,
                    }, { tradeHistoryBump: tradeHistoryBump, vaultOwnerBump: vaultOwnerBump })];
                case 1:
                    instruction = _c.sent();
                    tx = new web3_js_1.Transaction();
                    tx.add(instruction);
                    _b = tx;
                    return [4 /*yield*/, connection.getRecentBlockhash()];
                case 2:
                    _b.recentBlockhash = (_c.sent()).blockhash;
                    tx.feePayer = buyer;
                    return [2 /*return*/, { tx: tx }];
            }
        });
    });
};
exports.createBuyTransaction = createBuyTransaction;
