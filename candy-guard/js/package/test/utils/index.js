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
exports.getCandyGuardPDA = void 0;
__exportStar(require("./asserts"), exports);
__exportStar(require("./constants"), exports);
__exportStar(require("./errors"), exports);
const web3_js_1 = require("@solana/web3.js");
async function getCandyGuardPDA(programId, base) {
    return await web3_js_1.PublicKey.findProgramAddress([Buffer.from('candy_guard'), base.publicKey.toBuffer()], programId).then(result => { return result[0]; });
}
exports.getCandyGuardPDA = getCandyGuardPDA;
//# sourceMappingURL=index.js.map