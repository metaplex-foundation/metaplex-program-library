"use strict";
var __extends = (this && this.__extends) || (function () {
    var extendStatics = function (d, b) {
        extendStatics = Object.setPrototypeOf ||
            ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
            function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };
        return extendStatics(d, b);
    };
    return function (d, b) {
        if (typeof b !== "function" && b !== null)
            throw new TypeError("Class extends value " + String(b) + " is not a constructor or null");
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.FixedPriceSaleProgram = void 0;
var mpl_core_1 = require("@metaplex-foundation/mpl-core");
var web3_js_1 = require("@solana/web3.js");
var consts_1 = require("./consts");
var FixedPriceSaleProgram = /** @class */ (function (_super) {
    __extends(FixedPriceSaleProgram, _super);
    function FixedPriceSaleProgram() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    FixedPriceSaleProgram.PUBKEY = new web3_js_1.PublicKey(consts_1.PROGRAM_ID);
    return FixedPriceSaleProgram;
}(mpl_core_1.Program));
exports.FixedPriceSaleProgram = FixedPriceSaleProgram;
