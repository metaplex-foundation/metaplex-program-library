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
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.candyMachineBeet = exports.CandyMachine = exports.candyMachineDiscriminator = void 0;
const beet = __importStar(require("@metaplex-foundation/beet"));
const web3 = __importStar(require("@solana/web3.js"));
const beetSolana = __importStar(require("@metaplex-foundation/beet-solana"));
const CandyMachineData_1 = require("../types/CandyMachineData");
exports.candyMachineDiscriminator = [51, 173, 177, 113, 25, 241, 109, 189];
class CandyMachine {
    constructor(features, wallet, authority, updateAuthority, collectionMint, itemsRedeemed, data) {
        this.features = features;
        this.wallet = wallet;
        this.authority = authority;
        this.updateAuthority = updateAuthority;
        this.collectionMint = collectionMint;
        this.itemsRedeemed = itemsRedeemed;
        this.data = data;
    }
    static fromArgs(args) {
        return new CandyMachine(args.features, args.wallet, args.authority, args.updateAuthority, args.collectionMint, args.itemsRedeemed, args.data);
    }
    static fromAccountInfo(accountInfo, offset = 0) {
        return CandyMachine.deserialize(accountInfo.data, offset);
    }
    static async fromAccountAddress(connection, address) {
        const accountInfo = await connection.getAccountInfo(address);
        if (accountInfo == null) {
            throw new Error(`Unable to find CandyMachine account at ${address}`);
        }
        return CandyMachine.fromAccountInfo(accountInfo, 0)[0];
    }
    static gpaBuilder(programId = new web3.PublicKey('cndy3CZK71ZHMp9ddpq5NVvQDx33o6cCYDf4JBAWCk7')) {
        return beetSolana.GpaBuilder.fromStruct(programId, exports.candyMachineBeet);
    }
    static deserialize(buf, offset = 0) {
        return exports.candyMachineBeet.deserialize(buf, offset);
    }
    serialize() {
        return exports.candyMachineBeet.serialize({
            accountDiscriminator: exports.candyMachineDiscriminator,
            ...this,
        });
    }
    static byteSize(args) {
        const instance = CandyMachine.fromArgs(args);
        return exports.candyMachineBeet.toFixedFromValue({
            accountDiscriminator: exports.candyMachineDiscriminator,
            ...instance,
        }).byteSize;
    }
    static async getMinimumBalanceForRentExemption(args, connection, commitment) {
        return connection.getMinimumBalanceForRentExemption(CandyMachine.byteSize(args), commitment);
    }
    pretty() {
        return {
            features: (() => {
                const x = this.features;
                if (typeof x.toNumber === 'function') {
                    try {
                        return x.toNumber();
                    }
                    catch (_) {
                        return x;
                    }
                }
                return x;
            })(),
            wallet: this.wallet.toBase58(),
            authority: this.authority.toBase58(),
            updateAuthority: this.updateAuthority.toBase58(),
            collectionMint: this.collectionMint,
            itemsRedeemed: (() => {
                const x = this.itemsRedeemed;
                if (typeof x.toNumber === 'function') {
                    try {
                        return x.toNumber();
                    }
                    catch (_) {
                        return x;
                    }
                }
                return x;
            })(),
            data: this.data,
        };
    }
}
exports.CandyMachine = CandyMachine;
exports.candyMachineBeet = new beet.FixableBeetStruct([
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['features', beet.u64],
    ['wallet', beetSolana.publicKey],
    ['authority', beetSolana.publicKey],
    ['updateAuthority', beetSolana.publicKey],
    ['collectionMint', beet.coption(beetSolana.publicKey)],
    ['itemsRedeemed', beet.u64],
    ['data', CandyMachineData_1.candyMachineDataBeet],
], CandyMachine.fromArgs, 'CandyMachine');
//# sourceMappingURL=CandyMachine.js.map