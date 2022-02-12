"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.marketStateEnum = exports.MarketState = void 0;
var beet = require("@metaplex-foundation/beet");
var MarketState;
(function (MarketState) {
    MarketState[MarketState["Uninitialized"] = 0] = "Uninitialized";
    MarketState[MarketState["Created"] = 1] = "Created";
    MarketState[MarketState["Suspended"] = 2] = "Suspended";
    MarketState[MarketState["Active"] = 3] = "Active";
    MarketState[MarketState["Ended"] = 4] = "Ended";
})(MarketState = exports.MarketState || (exports.MarketState = {}));
exports.marketStateEnum = beet.fixedScalarEnum(MarketState);
