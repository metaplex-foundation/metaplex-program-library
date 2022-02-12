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
exports.createCreateStoreInstruction = void 0;
var web3 = require("@solana/web3.js");
var beet = require("@metaplex-foundation/beet");
var consts_1 = require("../consts");
var createStoreStruct = new beet.FixableBeetArgsStruct([
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['name', beet.utf8String],
    ['description', beet.utf8String],
], 'CreateStoreInstructionArgs');
var createStoreInstructionDiscriminator = [132, 152, 9, 27, 112, 19, 95, 83];
/**
 * Creates a _CreateStore_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
function createCreateStoreInstruction(accounts, args) {
    var admin = accounts.admin, store = accounts.store;
    var data = createStoreStruct.serialize(__assign({ instructionDiscriminator: createStoreInstructionDiscriminator }, args))[0];
    var keys = [
        {
            pubkey: admin,
            isWritable: true,
            isSigner: true,
        },
        {
            pubkey: store,
            isWritable: true,
            isSigner: true,
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
exports.createCreateStoreInstruction = createCreateStoreInstruction;
