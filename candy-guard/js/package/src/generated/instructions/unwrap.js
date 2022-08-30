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
exports.createUnwrapInstruction = exports.unwrapInstructionDiscriminator = exports.unwrapStruct = void 0;
const beet = __importStar(require("@metaplex-foundation/beet"));
const web3 = __importStar(require("@solana/web3.js"));
exports.unwrapStruct = new beet.BeetArgsStruct([['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]], 'UnwrapInstructionArgs');
exports.unwrapInstructionDiscriminator = [
    126, 175, 198, 14, 212, 69, 50, 44,
];
function createUnwrapInstruction(accounts, programId = new web3.PublicKey('grd1hVewsa8dR1T1JfSFGzQUqgWmc1xXZ3uRRFJJ8XJ')) {
    const [data] = exports.unwrapStruct.serialize({
        instructionDiscriminator: exports.unwrapInstructionDiscriminator,
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
exports.createUnwrapInstruction = createUnwrapInstruction;
//# sourceMappingURL=unwrap.js.map