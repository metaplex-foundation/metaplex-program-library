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
exports.createCreateSecondaryMetadataCreatorsInstruction = void 0;
var beet = require("@metaplex-foundation/beet");
var web3 = require("@solana/web3.js");
var consts_1 = require("../consts");
var Creator_1 = require("../accounts/Creator");
var createSecondaryMetadataCreatorsStruct = new beet.FixableBeetArgsStruct([
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['secondaryMetadataCreatorsBump', beet.u8],
    ['creators', beet.array(Creator_1.creatorAccountDataStruct)],
], 'CreateSecondaryMetadataCreatorsInstructionArgs');
var createSecondaryMetadataCreatorsInstructionDiscriminator = [
    179, 194, 135, 183, 65, 63, 241, 76,
];
/**
 * Creates a _CreateSecondaryMetadataCreators_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 */
function createCreateSecondaryMetadataCreatorsInstruction(accounts, args) {
    var admin = accounts.admin, metadata = accounts.metadata, secondaryMetadataCreators = accounts.secondaryMetadataCreators;
    var data = createSecondaryMetadataCreatorsStruct.serialize(__assign({ instructionDiscriminator: createSecondaryMetadataCreatorsInstructionDiscriminator }, args))[0];
    var keys = [
        {
            pubkey: admin,
            isWritable: true,
            isSigner: true,
        },
        {
            pubkey: metadata,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: secondaryMetadataCreators,
            isWritable: true,
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
exports.createCreateSecondaryMetadataCreatorsInstruction = createCreateSecondaryMetadataCreatorsInstruction;
