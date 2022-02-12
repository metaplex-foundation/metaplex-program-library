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
exports.createInitSellingResourceInstruction = void 0;
var web3 = require("@solana/web3.js");
var beet = require("@metaplex-foundation/beet");
var splToken = require("@solana/spl-token");
var consts_1 = require("../consts");
var initSellingResourceStruct = new beet.FixableBeetArgsStruct([
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['masterEditionBump', beet.u8],
    ['vaultOwnerBump', beet.u8],
    ['maxSupply', beet.coption(beet.u64)],
], 'InitSellingResourceInstructionArgs');
var initSellingResourceInstructionDiscriminator = [56, 15, 222, 211, 147, 205, 4, 145];
/**
 * Creates a _InitSellingResource_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
function createInitSellingResourceInstruction(accounts, args) {
    var store = accounts.store, admin = accounts.admin, sellingResource = accounts.sellingResource, sellingResourceOwner = accounts.sellingResourceOwner, resourceMint = accounts.resourceMint, masterEdition = accounts.masterEdition, metadata = accounts.metadata, vault = accounts.vault, owner = accounts.owner, resourceToken = accounts.resourceToken;
    var data = initSellingResourceStruct.serialize(__assign({ instructionDiscriminator: initSellingResourceInstructionDiscriminator }, args))[0];
    var keys = [
        {
            pubkey: store,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: admin,
            isWritable: true,
            isSigner: true,
        },
        {
            pubkey: sellingResource,
            isWritable: true,
            isSigner: true,
        },
        {
            pubkey: sellingResourceOwner,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: resourceMint,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: masterEdition,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: metadata,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: vault,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: owner,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: resourceToken,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: web3.SYSVAR_RENT_PUBKEY,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: splToken.TOKEN_PROGRAM_ID,
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
exports.createInitSellingResourceInstruction = createInitSellingResourceInstruction;
