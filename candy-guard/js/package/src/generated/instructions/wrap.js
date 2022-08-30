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
exports.createWrapInstruction = exports.wrapInstructionDiscriminator = exports.wrapStruct = void 0;
const beet = __importStar(require("@metaplex-foundation/beet"));
const web3 = __importStar(require("@solana/web3.js"));
exports.wrapStruct = new beet.BeetArgsStruct([['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]], 'WrapInstructionArgs');
exports.wrapInstructionDiscriminator = [
    178, 40, 10, 189, 228, 129, 186, 140,
];
function createWrapInstruction(accounts, programId = new web3.PublicKey('grd1hVewsa8dR1T1JfSFGzQUqgWmc1xXZ3uRRFJJ8XJ')) {
    const [data] = exports.wrapStruct.serialize({
        instructionDiscriminator: exports.wrapInstructionDiscriminator,
    });
    const keys = [
        {
            pubkey: accounts.candyGuard,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: accounts.candyMachine,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.candyMachineProgram,
            isWritable: false,
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
exports.createWrapInstruction = createWrapInstruction;
//# sourceMappingURL=wrap.js.map