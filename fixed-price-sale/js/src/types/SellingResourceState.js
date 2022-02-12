"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sellingResourceStateEnum = exports.SellingResourceState = void 0;
var beet = require("@metaplex-foundation/beet");
var SellingResourceState;
(function (SellingResourceState) {
    SellingResourceState[SellingResourceState["Uninitialized"] = 0] = "Uninitialized";
    SellingResourceState[SellingResourceState["Created"] = 1] = "Created";
    SellingResourceState[SellingResourceState["InUse"] = 2] = "InUse";
    SellingResourceState[SellingResourceState["Exhausted"] = 3] = "Exhausted";
    SellingResourceState[SellingResourceState["Stopped"] = 4] = "Stopped";
})(SellingResourceState = exports.SellingResourceState || (exports.SellingResourceState = {}));
exports.sellingResourceStateEnum = beet.fixedScalarEnum(SellingResourceState);
