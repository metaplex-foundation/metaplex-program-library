"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.assertIsNotNull = exports.spokSameBignum = exports.spokSamePubkey = exports.assertSamePubkey = void 0;
const bn_js_1 = __importDefault(require("bn.js"));
function assertSamePubkey(t, a, b) {
    t.equal(a === null || a === void 0 ? void 0 : a.toBase58(), b.toBase58(), 'pubkeys are same');
}
exports.assertSamePubkey = assertSamePubkey;
function spokSamePubkey(a) {
    const same = (b) => b != null && !!(a === null || a === void 0 ? void 0 : a.equals(b));
    same.$spec = `spokSamePubkey(${a === null || a === void 0 ? void 0 : a.toBase58()})`;
    same.$description = `${a === null || a === void 0 ? void 0 : a.toBase58()} equal`;
    return same;
}
exports.spokSamePubkey = spokSamePubkey;
function spokSameBignum(a) {
    const same = (b) => b != null && new bn_js_1.default(a).eq(new bn_js_1.default(b));
    same.$spec = `spokSameBignum(${a})`;
    same.$description = `${a} equal`;
    return same;
}
exports.spokSameBignum = spokSameBignum;
function assertIsNotNull(t, x) {
    t.ok(x, 'should be non null');
}
exports.assertIsNotNull = assertIsNotNull;
//# sourceMappingURL=asserts.js.map