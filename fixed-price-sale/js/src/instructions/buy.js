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
exports.createBuyInstruction = void 0;
var splToken = require("@solana/spl-token");
var beet = require("@metaplex-foundation/beet");
var web3 = require("@solana/web3.js");
var consts_1 = require("../consts");
var buyStruct = new beet.BeetArgsStruct([
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['tradeHistoryBump', beet.u8],
    ['vaultOwnerBump', beet.u8],
], 'BuyInstructionArgs');
var buyInstructionDiscriminator = [102, 6, 61, 18, 1, 218, 235, 234];
/**
 * Creates a _Buy_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
function createBuyInstruction(accounts, args) {
    var market = accounts.market, sellingResource = accounts.sellingResource, userTokenAccount = accounts.userTokenAccount, userWallet = accounts.userWallet, tradeHistory = accounts.tradeHistory, treasuryHolder = accounts.treasuryHolder, newMetadata = accounts.newMetadata, newEdition = accounts.newEdition, masterEdition = accounts.masterEdition, newMint = accounts.newMint, editionMarker = accounts.editionMarker, vault = accounts.vault, owner = accounts.owner, masterEditionMetadata = accounts.masterEditionMetadata, clock = accounts.clock, tokenMetadataProgram = accounts.tokenMetadataProgram;
    var data = buyStruct.serialize(__assign({ instructionDiscriminator: buyInstructionDiscriminator }, args))[0];
    var keys = [
        {
            pubkey: market,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: sellingResource,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: userTokenAccount,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: userWallet,
            isWritable: false,
            isSigner: true,
        },
        {
            pubkey: tradeHistory,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: treasuryHolder,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: newMetadata,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: newEdition,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: masterEdition,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: newMint,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: editionMarker,
            isWritable: true,
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
            pubkey: masterEditionMetadata,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: clock,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: web3.SYSVAR_RENT_PUBKEY,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: tokenMetadataProgram,
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
exports.createBuyInstruction = createBuyInstruction;
