"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const tape_1 = __importDefault(require("tape"));
const spok_1 = __importDefault(require("spok"));
const setup_1 = require("./setup/");
const generated_1 = require("../src/generated");
const utils_1 = require("./utils");
const init = new setup_1.InitTransactions();
(0, setup_1.killStuckProcess)();
(0, tape_1.default)('initialize: new candy machine', async (t) => {
    const { fstTxHandler, payerPair, connection } = await init.payer();
    const items = 10;
    const data = {
        itemsAvailable: items,
        symbol: 'CORE',
        sellerFeeBasisPoints: 500,
        maxSupply: 0,
        isMutable: true,
        retainAuthority: true,
        creators: [{
                address: payerPair.publicKey,
                verified: false,
                percentageShare: 100
            }],
        configLineSettings: {
            prefixName: 'TEST ',
            nameLength: 10,
            prefixUri: 'https://arweave.net/',
            uriLength: 50,
            isSequential: false
        },
        hiddenSettings: null
    };
    const { tx: transaction, candyMachine: address } = await init.create(t, payerPair, data, fstTxHandler, connection);
    await transaction.assertSuccess(t);
    const candyMachine = await generated_1.CandyMachine.fromAccountAddress(connection, address);
    (0, spok_1.default)(t, candyMachine, {
        authority: (0, utils_1.spokSamePubkey)(payerPair.publicKey),
        updateAuthority: (0, utils_1.spokSamePubkey)(payerPair.publicKey),
        itemsRedeemed: (0, utils_1.spokSameBignum)(0),
        data: {
            itemsAvailable: (0, utils_1.spokSameBignum)(items),
            maxSupply: (0, utils_1.spokSameBignum)(0),
            isMutable: true,
            retainAuthority: true,
            creators: data.creators,
            configLineSettings: data.configLineSettings
        }
    });
    t.notOk(candyMachine.data.hiddenSettings, 'hidden settings should be null');
});
(0, tape_1.default)('initialize: new candy machine (hidden settings)', async (t) => {
    const { fstTxHandler, payerPair, connection } = await init.payer();
    const items = 100;
    const data = {
        itemsAvailable: items,
        symbol: 'CORE',
        sellerFeeBasisPoints: 500,
        maxSupply: 0,
        isMutable: true,
        retainAuthority: true,
        creators: [{
                address: payerPair.publicKey,
                verified: false,
                percentageShare: 100
            }],
        configLineSettings: null,
        hiddenSettings: {
            name: 'Hidden NFT',
            uri: 'https://arweave.net/uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ',
            hash: Buffer.from('74bac30d82a0baa41dd2bee4b41bbc36').toJSON().data
        }
    };
    const { tx: transaction, candyMachine: address } = await init.create(t, payerPair, data, fstTxHandler, connection);
    await transaction.assertSuccess(t);
    const candyMachine = await generated_1.CandyMachine.fromAccountAddress(connection, address);
    (0, spok_1.default)(t, candyMachine, {
        authority: (0, utils_1.spokSamePubkey)(payerPair.publicKey),
        updateAuthority: (0, utils_1.spokSamePubkey)(payerPair.publicKey),
        itemsRedeemed: (0, utils_1.spokSameBignum)(0),
        data: {
            itemsAvailable: (0, utils_1.spokSameBignum)(items),
            maxSupply: (0, utils_1.spokSameBignum)(0),
            isMutable: true,
            retainAuthority: true,
            creators: data.creators,
            hiddenSettings: data.hiddenSettings
        }
    });
    ;
    t.notOk(candyMachine.data.configLineSettings, 'config lines settings should be null');
});
//# sourceMappingURL=01-initialize.js.map