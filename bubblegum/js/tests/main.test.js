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
var web3_js_1 = require("@solana/web3.js");
var spl_account_compression_1 = require("@solana/spl-account-compression");
var generated_1 = require("../src/generated");
var mpl_bubblegum_1 = require("../src/mpl-bubblegum");
var bn_js_1 = require("bn.js");
function keypairFromSeed(seed) {
    var expandedSeed = Uint8Array.from(Buffer.from("".concat(seed, "                                           ")));
    return web3_js_1.Keypair.fromSeed(expandedSeed.slice(0, 32));
}
function makeCompressedNFT(name, symbol, creators) {
    if (creators === void 0) { creators = []; }
    return {
        name: name,
        symbol: symbol,
        uri: 'https://metaplex.com',
        creators: creators,
        editionNonce: 0,
        tokenProgramVersion: generated_1.TokenProgramVersion.Original,
        tokenStandard: generated_1.TokenStandard.Fungible,
        uses: null,
        collection: null,
        primarySaleHappened: false,
        sellerFeeBasisPoints: 0,
        isMutable: false,
    };
}
function setupTreeWithCompressedNFT(connection, payerKeypair, compressedNFT, maxDepth, maxBufferSize) {
    if (maxDepth === void 0) { maxDepth = 14; }
    if (maxBufferSize === void 0) { maxBufferSize = 64; }
    return __awaiter(this, void 0, void 0, function () {
        var payer, merkleTreeKeypair, merkleTree, space, allocTreeIx, _a, _b, _c, treeAuthority, _bump, createTreeIx, mintIx, tx;
        var _d;
        return __generator(this, function (_e) {
            switch (_e.label) {
                case 0:
                    payer = payerKeypair.publicKey;
                    merkleTreeKeypair = web3_js_1.Keypair.generate();
                    merkleTree = merkleTreeKeypair.publicKey;
                    space = (0, spl_account_compression_1.getConcurrentMerkleTreeAccountSize)(maxDepth, maxBufferSize);
                    _b = (_a = web3_js_1.SystemProgram).createAccount;
                    _d = {
                        fromPubkey: payer,
                        newAccountPubkey: merkleTree
                    };
                    return [4 /*yield*/, connection.getMinimumBalanceForRentExemption(space)];
                case 1:
                    allocTreeIx = _b.apply(_a, [(_d.lamports = _e.sent(),
                            _d.space = space,
                            _d.programId = spl_account_compression_1.SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
                            _d)]);
                    return [4 /*yield*/, web3_js_1.PublicKey.findProgramAddress([merkleTree.toBuffer()], generated_1.PROGRAM_ID)];
                case 2:
                    _c = _e.sent(), treeAuthority = _c[0], _bump = _c[1];
                    createTreeIx = (0, generated_1.createCreateTreeInstruction)({
                        merkleTree: merkleTree,
                        treeAuthority: treeAuthority,
                        treeCreator: payer,
                        payer: payer,
                        logWrapper: spl_account_compression_1.SPL_NOOP_PROGRAM_ID,
                        compressionProgram: spl_account_compression_1.SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
                    }, {
                        maxBufferSize: maxBufferSize,
                        maxDepth: maxDepth,
                        public: false,
                    }, generated_1.PROGRAM_ID);
                    mintIx = (0, generated_1.createMintV1Instruction)({
                        merkleTree: merkleTree,
                        treeAuthority: treeAuthority,
                        treeDelegate: payer,
                        payer: payer,
                        leafDelegate: payer,
                        leafOwner: payer,
                        compressionProgram: spl_account_compression_1.SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
                        logWrapper: spl_account_compression_1.SPL_NOOP_PROGRAM_ID,
                    }, {
                        message: compressedNFT,
                    });
                    tx = new web3_js_1.Transaction().add(allocTreeIx).add(createTreeIx).add(mintIx);
                    tx.feePayer = payer;
                    return [4 /*yield*/, (0, web3_js_1.sendAndConfirmTransaction)(connection, tx, [merkleTreeKeypair, payerKeypair], {
                            commitment: 'confirmed',
                            skipPreflight: true,
                        })];
                case 3:
                    _e.sent();
                    return [2 /*return*/, {
                            merkleTree: merkleTree,
                        }];
            }
        });
    });
}
describe('Bubblegum tests', function () {
    var connection = new web3_js_1.Connection('http://localhost:8899');
    var payerKeypair = keypairFromSeed('metaplex-test');
    var payer = payerKeypair.publicKey;
    beforeEach(function () { return __awaiter(void 0, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, connection.requestAirdrop(payer, web3_js_1.LAMPORTS_PER_SOL)];
                case 1:
                    _a.sent();
                    return [2 /*return*/];
            }
        });
    }); });
    it('Can create a Bubblegum tree and mint to it', function () { return __awaiter(void 0, void 0, void 0, function () {
        var compressedNFT;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    compressedNFT = {
                        name: 'Test Compressed NFT',
                        symbol: 'TST',
                        uri: 'https://metaplex.com',
                        creators: [],
                        editionNonce: 0,
                        tokenProgramVersion: generated_1.TokenProgramVersion.Original,
                        tokenStandard: generated_1.TokenStandard.Fungible,
                        uses: null,
                        collection: null,
                        primarySaleHappened: false,
                        sellerFeeBasisPoints: 0,
                        isMutable: false,
                    };
                    return [4 /*yield*/, setupTreeWithCompressedNFT(connection, payerKeypair, compressedNFT, 14, 64)];
                case 1:
                    _a.sent();
                    return [2 /*return*/];
            }
        });
    }); });
    describe('Unit test compressed NFT instructions', function () {
        var merkleTree;
        var originalCompressedNFT = makeCompressedNFT('test', 'TST');
        beforeEach(function () { return __awaiter(void 0, void 0, void 0, function () {
            var result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, connection.requestAirdrop(payer, web3_js_1.LAMPORTS_PER_SOL)];
                    case 1:
                        _a.sent();
                        return [4 /*yield*/, setupTreeWithCompressedNFT(connection, payerKeypair, originalCompressedNFT, 14, 64)];
                    case 2:
                        result = _a.sent();
                        merkleTree = result.merkleTree;
                        return [2 /*return*/];
                }
            });
        }); });
        it('Can verify existence a compressed NFT', function () { return __awaiter(void 0, void 0, void 0, function () {
            var accountInfo, account, leafIndex, assetId, verifyLeafIx, tx, txId;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, connection.getAccountInfo(merkleTree, { commitment: 'confirmed' })];
                    case 1:
                        accountInfo = _a.sent();
                        account = spl_account_compression_1.ConcurrentMerkleTreeAccount.fromBuffer(accountInfo.data);
                        leafIndex = new bn_js_1.BN.BN(0);
                        return [4 /*yield*/, (0, mpl_bubblegum_1.getLeafAssetId)(merkleTree, leafIndex)];
                    case 2:
                        assetId = _a.sent();
                        verifyLeafIx = (0, spl_account_compression_1.createVerifyLeafIx)(merkleTree, account.getCurrentRoot(), (0, mpl_bubblegum_1.computeCompressedNFTHash)(assetId, payer, payer, leafIndex, originalCompressedNFT), 0, []);
                        tx = new web3_js_1.Transaction().add(verifyLeafIx);
                        return [4 /*yield*/, (0, web3_js_1.sendAndConfirmTransaction)(connection, tx, [payerKeypair], {
                                commitment: 'confirmed',
                                skipPreflight: true,
                            })];
                    case 3:
                        txId = _a.sent();
                        console.log('Verified NFT existence:', txId);
                        return [2 /*return*/];
                }
            });
        }); });
        // TODO(@metaplex): add collection tests here
    });
});
