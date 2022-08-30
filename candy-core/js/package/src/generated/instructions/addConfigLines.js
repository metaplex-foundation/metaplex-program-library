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
exports.createAddConfigLinesInstruction = exports.addConfigLinesInstructionDiscriminator = exports.addConfigLinesStruct = void 0;
const beet = __importStar(require("@metaplex-foundation/beet"));
const web3 = __importStar(require("@solana/web3.js"));
const ConfigLine_1 = require("../types/ConfigLine");
exports.addConfigLinesStruct = new beet.FixableBeetArgsStruct([
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['index', beet.u32],
    ['configLines', beet.array(ConfigLine_1.configLineBeet)],
], 'AddConfigLinesInstructionArgs');
exports.addConfigLinesInstructionDiscriminator = [
    223, 50, 224, 227, 151, 8, 115, 106,
];
function createAddConfigLinesInstruction(accounts, args, programId = new web3.PublicKey('cndy3CZK71ZHMp9ddpq5NVvQDx33o6cCYDf4JBAWCk7')) {
    const [data] = exports.addConfigLinesStruct.serialize({
        instructionDiscriminator: exports.addConfigLinesInstructionDiscriminator,
        ...args,
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
    ];
    const ix = new web3.TransactionInstruction({
        programId,
        keys,
        data,
    });
    return ix;
}
exports.createAddConfigLinesInstruction = createAddConfigLinesInstruction;
//# sourceMappingURL=addConfigLines.js.map