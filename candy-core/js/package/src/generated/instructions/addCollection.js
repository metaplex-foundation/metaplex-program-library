"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.createAddCollectionInstruction = exports.addCollectionInstructionDiscriminator = exports.addCollectionStruct = void 0;
const beet = __importStar(require("@metaplex-foundation/beet"));
const web3 = __importStar(require("@solana/web3.js"));
exports.addCollectionStruct = new beet.BeetArgsStruct([['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]], 'AddCollectionInstructionArgs');
exports.addCollectionInstructionDiscriminator = [
    79, 172, 225, 142, 219, 192, 171, 80,
];
function createAddCollectionInstruction(accounts, programId = new web3.PublicKey('cndy3CZK71ZHMp9ddpq5NVvQDx33o6cCYDf4JBAWCk7')) {
    var _a, _b;
    const [data] = exports.addCollectionStruct.serialize({
        instructionDiscriminator: exports.addCollectionInstructionDiscriminator,
    });
    const keys = [
        {
            pubkey: accounts.candyMachine,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.authority,
            isWritable: false,
            isSigner: true,
        },
        {
            pubkey: accounts.updateAuthority,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: accounts.payer,
            isWritable: false,
            isSigner: true,
        },
        {
            pubkey: accounts.collectionAuthority,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: accounts.collectionMetadata,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: accounts.collectionMint,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: accounts.collectionEdition,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: accounts.collectionAuthorityRecord,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.tokenMetadataProgram,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: (_a = accounts.systemProgram) !== null && _a !== void 0 ? _a : web3.SystemProgram.programId,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: (_b = accounts.rent) !== null && _b !== void 0 ? _b : web3.SYSVAR_RENT_PUBKEY,
            isWritable: false,
            isSigner: false,
        },
    ];
    const ix = new web3.TransactionInstruction({
        programId,
        keys,
        data,
    });
    return ix;
}
exports.createAddCollectionInstruction = createAddCollectionInstruction;
//# sourceMappingURL=addCollection.js.map