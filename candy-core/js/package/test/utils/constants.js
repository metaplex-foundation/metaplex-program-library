"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.HIDDEN_SECTION = exports.MAX_CREATOR_LIMIT = exports.MAX_CREATOR_LEN = exports.MAX_SYMBOL_LENGTH = exports.MAX_URI_LENGTH = exports.MAX_NAME_LENGTH = void 0;
exports.MAX_NAME_LENGTH = 32;
exports.MAX_URI_LENGTH = 200;
exports.MAX_SYMBOL_LENGTH = 10;
exports.MAX_CREATOR_LEN = 32 + 1 + 1;
exports.MAX_CREATOR_LIMIT = 5;
exports.HIDDEN_SECTION = 8
    + 8
    + 32
    + 32
    + 32
    + 33
    + 8
    + 8
    + 4 + exports.MAX_SYMBOL_LENGTH
    + 2
    + 8
    + 1
    + 1
    + 4 + exports.MAX_CREATOR_LIMIT * exports.MAX_CREATOR_LEN
    + 1
    + 4 + exports.MAX_NAME_LENGTH
    + 4
    + 4 + exports.MAX_URI_LENGTH
    + 4
    + 1
    + 1
    + 4 + exports.MAX_NAME_LENGTH
    + 4 + exports.MAX_URI_LENGTH
    + 32;
//# sourceMappingURL=constants.js.map