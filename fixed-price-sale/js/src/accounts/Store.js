"use strict";
var __assign = (this && this.__assign) || function () {
    __assign = Object.assign || function(t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
            s = arguments[i];
            for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
                t[p] = s[p];
        }
        return t;
    };
    return __assign.apply(this, arguments);
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.StoreAccountData = void 0;
var beetSolana = require("@metaplex-foundation/beet-solana");
var beet = require("@metaplex-foundation/beet");
var storeAccountDiscriminator = [130, 48, 247, 244, 182, 191, 30, 26];
/**
 * Holds the data for the {@link StoreAccount} and provides de/serialization
 * functionality for that data
 */
var StoreAccountData = /** @class */ (function () {
    function StoreAccountData(admin, name, description) {
        this.admin = admin;
        this.name = name;
        this.description = description;
    }
    /**
     * Creates a {@link StoreAccountData} instance from the provided args.
     */
    StoreAccountData.fromArgs = function (args) {
        return new StoreAccountData(args.admin, args.name, args.description);
    };
    /**
     * Deserializes the {@link StoreAccountData} from the data of the provided {@link web3.AccountInfo}.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    StoreAccountData.fromAccountInfo = function (accountInfo, offset) {
        if (offset === void 0) { offset = 0; }
        return StoreAccountData.deserialize(accountInfo.data, offset);
    };
    /**
     * Deserializes the {@link StoreAccountData} from the provided data Buffer.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    StoreAccountData.deserialize = function (buf, offset) {
        if (offset === void 0) { offset = 0; }
        return storeAccountDataStruct.deserialize(buf, offset);
    };
    /**
     * Returns the byteSize of a {@link Buffer} holding the serialized data of
     * {@link StoreAccountData} for the provided args.
     *
     * @param args need to be provided since the byte size for this account
     * depends on them
     */
    StoreAccountData.byteSize = function (args) {
        var instance = StoreAccountData.fromArgs(args);
        return storeAccountDataStruct.toFixedFromValue(__assign({ accountDiscriminator: storeAccountDiscriminator }, instance)).byteSize;
    };
    /**
     * Fetches the minimum balance needed to exempt an account holding
     * {@link StoreAccountData} data from rent
     *
     * @param args need to be provided since the byte size for this account
     * depends on them
     * @param connection used to retrieve the rent exemption information
     */
    StoreAccountData.getMinimumBalanceForRentExemption = function (args, connection, commitment) {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                return [2 /*return*/, connection.getMinimumBalanceForRentExemption(StoreAccountData.byteSize(args), commitment)];
            });
        });
    };
    /**
     * Serializes the {@link StoreAccountData} into a Buffer.
     * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
     */
    StoreAccountData.prototype.serialize = function () {
        return storeAccountDataStruct.serialize(__assign({ accountDiscriminator: storeAccountDiscriminator }, this));
    };
    /**
     * Returns a readable version of {@link StoreAccountData} properties
     * and can be used to convert to JSON and/or logging
     */
    StoreAccountData.prototype.pretty = function () {
        return {
            admin: this.admin.toBase58(),
            name: this.name,
            description: this.description,
        };
    };
    return StoreAccountData;
}());
exports.StoreAccountData = StoreAccountData;
var storeAccountDataStruct = new beet.FixableBeetStruct([
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['admin', beetSolana.publicKey],
    ['name', beet.utf8String],
    ['description', beet.utf8String],
], StoreAccountData.fromArgs, 'StoreAccountData');
