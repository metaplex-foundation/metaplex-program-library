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
exports.mintNFT = void 0;
var web3_js_1 = require("@solana/web3.js");
var amman_1 = require("@metaplex-foundation/amman");
var mpl_token_metadata_1 = require("@metaplex-foundation/mpl-token-metadata");
var spl_token_1 = require("@solana/spl-token");
var assert_1 = require("assert");
var createTokenAccount_1 = require("../transactions/createTokenAccount");
var createMetadata_1 = require("./createMetadata");
var URI = 'https://arweave.net/Rmg4pcIv-0FQ7M7X838p2r592Q4NU63Fj7o7XsvBHEE';
var NAME = 'test';
var SYMBOL = 'sym';
var SELLER_FEE_BASIS_POINTS = 10;
function mintNFT(_a) {
    var transactionHandler = _a.transactionHandler, payer = _a.payer, connection = _a.connection, _b = _a.creators, creators = _b === void 0 ? null : _b;
    return __awaiter(this, void 0, void 0, function () {
        var _c, mint, createMintTx, mintRes, _d, tokenAccount, createTokenTx, associatedTokenAccountRes, initMetadataData, createTxDetails, metadataPDA, _e, edition, editionBump, masterEditionTx, masterEditionRes;
        return __generator(this, function (_f) {
            switch (_f.label) {
                case 0: return [4 /*yield*/, new amman_1.Actions(connection).createMintAccount(payer.publicKey)];
                case 1:
                    _c = _f.sent(), mint = _c.mint, createMintTx = _c.createMintTx;
                    return [4 /*yield*/, transactionHandler.sendAndConfirmTransaction(createMintTx, [mint], amman_1.defaultSendOptions)];
                case 2:
                    mintRes = _f.sent();
                    (0, amman_1.assertConfirmedTransaction)(assert_1.strict, mintRes.txConfirmed);
                    return [4 /*yield*/, (0, createTokenAccount_1.createTokenAccount)({
                            payer: payer.publicKey,
                            mint: mint.publicKey,
                            connection: connection,
                        })];
                case 3:
                    _d = _f.sent(), tokenAccount = _d.tokenAccount, createTokenTx = _d.createTokenTx;
                    createTokenTx.add(spl_token_1.Token.createMintToInstruction(new web3_js_1.PublicKey(spl_token_1.TOKEN_PROGRAM_ID), mint.publicKey, tokenAccount.publicKey, payer.publicKey, [], 1));
                    return [4 /*yield*/, transactionHandler.sendAndConfirmTransaction(createTokenTx, [tokenAccount], amman_1.defaultSendOptions)];
                case 4:
                    associatedTokenAccountRes = _f.sent();
                    (0, amman_1.assertConfirmedTransaction)(assert_1.strict, associatedTokenAccountRes.txConfirmed);
                    initMetadataData = new mpl_token_metadata_1.MetadataDataData({
                        uri: URI,
                        name: NAME,
                        symbol: SYMBOL,
                        sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
                        creators: creators,
                    });
                    return [4 /*yield*/, (0, createMetadata_1.createMetadata)({
                            transactionHandler: transactionHandler,
                            publicKey: payer.publicKey,
                            editionMint: mint.publicKey,
                            metadataData: initMetadataData,
                        })];
                case 5:
                    createTxDetails = (_f.sent()).createTxDetails;
                    (0, amman_1.assertConfirmedTransaction)(assert_1.strict, createTxDetails.txConfirmed);
                    return [4 /*yield*/, mpl_token_metadata_1.Metadata.getPDA(mint.publicKey)];
                case 6:
                    metadataPDA = _f.sent();
                    return [4 /*yield*/, web3_js_1.PublicKey.findProgramAddress([
                            Buffer.from(mpl_token_metadata_1.MetadataProgram.PREFIX),
                            mpl_token_metadata_1.MetadataProgram.PUBKEY.toBuffer(),
                            new web3_js_1.PublicKey(mint.publicKey).toBuffer(),
                            Buffer.from(mpl_token_metadata_1.MasterEdition.EDITION_PREFIX),
                        ], mpl_token_metadata_1.MetadataProgram.PUBKEY)];
                case 7:
                    _e = _f.sent(), edition = _e[0], editionBump = _e[1];
                    masterEditionTx = new mpl_token_metadata_1.CreateMasterEdition({ feePayer: payer.publicKey }, {
                        edition: edition,
                        metadata: metadataPDA,
                        updateAuthority: payer.publicKey,
                        mint: mint.publicKey,
                        mintAuthority: payer.publicKey,
                    });
                    return [4 /*yield*/, transactionHandler.sendAndConfirmTransaction(masterEditionTx, [], {
                            skipPreflight: false,
                        })];
                case 8:
                    masterEditionRes = _f.sent();
                    (0, amman_1.assertConfirmedTransaction)(assert_1.strict, masterEditionRes.txConfirmed);
                    return [2 /*return*/, { tokenAccount: tokenAccount, edition: edition, editionBump: editionBump, mint: mint, metadata: metadataPDA }];
            }
        });
    });
}
exports.mintNFT = mintNFT;
