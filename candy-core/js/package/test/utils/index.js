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
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getCandyMachineSpace = exports.getCandyMachinePDA = void 0;
__exportStar(require("./asserts"), exports);
__exportStar(require("./constants"), exports);
__exportStar(require("./errors"), exports);
const web3_js_1 = require("@solana/web3.js");
const constants_1 = require("./constants");
async function getCandyMachinePDA(programId, base) {
    return await web3_js_1.PublicKey.findProgramAddress([Buffer.from('candy_machine'), base.publicKey.toBuffer()], programId).then(result => { return result[0]; });
}
exports.getCandyMachinePDA = getCandyMachinePDA;
function getCandyMachineSpace(data) {
    if (data.configLineSettings == null) {
        return constants_1.HIDDEN_SECTION;
    }
    else {
        const items = parseInt(data.itemsAvailable.toString());
        return constants_1.HIDDEN_SECTION
            + 4
            + items * (data.configLineSettings.nameLength + data.configLineSettings.uriLength)
            + 4
            + (Math.floor(items / 8) + 1)
            + 4
            + items * 4;
    }
}
exports.getCandyMachineSpace = getCandyMachineSpace;
//# sourceMappingURL=index.js.map