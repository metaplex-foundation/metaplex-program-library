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
exports.PROGRAM_ID = exports.PROGRAM_ADDRESS = void 0;
const web3_js_1 = require("@solana/web3.js");
__exportStar(require("./accounts"), exports);
__exportStar(require("./errors"), exports);
__exportStar(require("./instructions"), exports);
__exportStar(require("./types"), exports);
exports.PROGRAM_ADDRESS = 'cndy3CZK71ZHMp9ddpq5NVvQDx33o6cCYDf4JBAWCk7';
exports.PROGRAM_ID = new web3_js_1.PublicKey(exports.PROGRAM_ADDRESS);
//# sourceMappingURL=index.js.map