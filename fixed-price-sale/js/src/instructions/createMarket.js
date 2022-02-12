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
Object.defineProperty(exports, "__esModule", { value: true });
exports.createCreateMarketInstruction = void 0;
var web3 = require("@solana/web3.js");
var beet = require("@metaplex-foundation/beet");
var consts_1 = require("../consts");
var createMarketStruct = new beet.FixableBeetArgsStruct([
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['treasuryOwnerBump', beet.u8],
    ['name', beet.utf8String],
    ['description', beet.utf8String],
    ['mutable', beet.bool],
    ['price', beet.u64],
    ['piecesInOneWallet', beet.coption(beet.u64)],
    ['startDate', beet.u64],
    ['endDate', beet.coption(beet.u64)],
], 'CreateMarketInstructionArgs');
var createMarketInstructionDiscriminator = [103, 226, 97, 235, 200, 188, 251, 254];
/**
 * Creates a _CreateMarket_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
function createCreateMarketInstruction(accounts, args) {
    var market = accounts.market, store = accounts.store, sellingResourceOwner = accounts.sellingResourceOwner, sellingResource = accounts.sellingResource, mint = accounts.mint, treasuryHolder = accounts.treasuryHolder, owner = accounts.owner;
    var data = createMarketStruct.serialize(__assign({ instructionDiscriminator: createMarketInstructionDiscriminator }, args))[0];
    var keys = [
        {
            pubkey: market,
            isWritable: true,
            isSigner: true,
        },
        {
            pubkey: store,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: sellingResourceOwner,
            isWritable: true,
            isSigner: true,
        },
        {
            pubkey: sellingResource,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: mint,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: treasuryHolder,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: owner,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: web3.SystemProgram.programId,
            isWritable: false,
            isSigner: false,
        },
    ];
    var ix = new web3.TransactionInstruction({
        programId: new web3.PublicKey(consts_1.PROGRAM_ID),
        keys: keys,
        data: data,
    });
    return ix;
}
exports.createCreateMarketInstruction = createCreateMarketInstruction;
