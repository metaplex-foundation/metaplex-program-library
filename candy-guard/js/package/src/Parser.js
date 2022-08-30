"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseData = void 0;
const types_1 = require("./generated/types");
const bn_js_1 = require("bn.js");
function parseData(candyGuard, buffer) {
    var _a, _b, _c, _d, _e, _f, _g, _h;
    const enabled = new bn_js_1.BN(candyGuard.features).toNumber();
    let current = 0;
    let data = {};
    let feature = 1;
    if (feature & enabled) {
        const [botTax, offset] = types_1.botTaxBeet.deserialize(buffer, current);
        data['botTax'] = botTax;
        current = offset;
    }
    feature = feature << 1;
    if (feature & enabled) {
        const [liveDate, offset] = types_1.liveDateBeet.deserialize(buffer, current);
        data['liveDate'] = liveDate;
        current = offset;
    }
    feature = feature << 1;
    if (feature & enabled) {
        const [lamports, offset] = types_1.lamportsBeet.deserialize(buffer, current);
        data['lamports'] = lamports;
        current = offset;
    }
    feature = feature << 1;
    if (feature & enabled) {
        const [splToken, offset] = types_1.splTokenBeet.deserialize(buffer, current);
        data['splToken'] = splToken;
        current = offset;
    }
    feature = feature << 1;
    if (feature & enabled) {
        const [thirdPartySigner, offset] = types_1.thirdPartySignerBeet.deserialize(buffer, current);
        data['thirdPartySigner'] = thirdPartySigner;
        current = offset;
    }
    feature = feature << 1;
    if (feature & enabled) {
        const [whitelist, offset] = types_1.whitelistBeet.deserialize(buffer, current);
        data['whitelist'] = whitelist;
        current = offset;
    }
    feature = feature << 1;
    if (feature & enabled) {
        const [gatekeeper, offset] = types_1.gatekeeperBeet.deserialize(buffer, current);
        data['gatekeeper'] = gatekeeper;
        current = offset;
    }
    feature = feature << 1;
    if (feature & enabled) {
        const [endSettings, offset] = types_1.endSettingsBeet.deserialize(buffer, current);
        data['endSettings'] = endSettings;
        current = offset;
    }
    return {
        botTax: (_a = data['botTax']) !== null && _a !== void 0 ? _a : null,
        liveDate: (_b = data['liveDate']) !== null && _b !== void 0 ? _b : null,
        lamports: (_c = data['lamports']) !== null && _c !== void 0 ? _c : null,
        splToken: (_d = data['splToken']) !== null && _d !== void 0 ? _d : null,
        thirdPartySigner: (_e = data['thirdPartySigner']) !== null && _e !== void 0 ? _e : null,
        whitelist: (_f = data['whitelist']) !== null && _f !== void 0 ? _f : null,
        gatekeeper: (_g = data['gateKeeper']) !== null && _g !== void 0 ? _g : null,
        endSettings: (_h = data['endSettings']) !== null && _h !== void 0 ? _h : null
    };
}
exports.parseData = parseData;
//# sourceMappingURL=Parser.js.map