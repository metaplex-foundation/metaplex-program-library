"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const tape_1 = __importDefault(require("tape"));
const spok_1 = __importDefault(require("spok"));
const generated_1 = require("src/generated");
const setup_1 = require("./setup/");
const utils_1 = require("./utils");
const init = new setup_1.InitTransactions();
(0, setup_1.killStuckProcess)();
(0, tape_1.default)('update', async (t) => {
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
    (0, spok_1.default)(t, candyMachine.data, {
        sellerFeeBasisPoints: 500,
        isMutable: true,
        retainAuthority: true,
        configLineSettings: {
            prefixName: 'TEST ',
            nameLength: 10,
            prefixUri: 'https://arweave.net/',
            uriLength: 50
        }
    });
    data.sellerFeeBasisPoints = 1000;
    data.isMutable = false;
    data.retainAuthority = false;
    data.configLineSettings.nameLength = 5;
    data.configLineSettings.uriLength = 25;
    const { tx: updateTransaction1 } = await init.updateCandyMachine(t, address, payerPair.publicKey, payerPair, data, fstTxHandler);
    await updateTransaction1.assertSuccess(t);
    const updatedCandyMachine = await generated_1.CandyMachine.fromAccountAddress(connection, address);
    (0, spok_1.default)(t, updatedCandyMachine.data, {
        sellerFeeBasisPoints: 1000,
        isMutable: false,
        retainAuthority: false,
        configLineSettings: data.configLineSettings
    });
    data.configLineSettings.nameLength = 15;
    data.configLineSettings.uriLength = 100;
    const { tx: updateTransaction2 } = await init.updateCandyMachine(t, address, payerPair.publicKey, payerPair, data, fstTxHandler);
    await updateTransaction2.assertError(t);
    data.itemsAvailable = 100;
    data.configLineSettings.nameLength = 5;
    data.configLineSettings.uriLength = 10;
    const { tx: updateTransaction3 } = await init.updateCandyMachine(t, address, payerPair.publicKey, payerPair, data, fstTxHandler);
    await updateTransaction3.assertError(t);
});
(0, tape_1.default)('update (hidden settings)', async (t) => {
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
    (0, spok_1.default)(t, candyMachine.data, {
        sellerFeeBasisPoints: 500,
        isMutable: true,
        retainAuthority: true,
        hiddenSettings: data.hiddenSettings
    });
    data.itemsAvailable = 1000;
    const { tx: updateTransaction1 } = await init.updateCandyMachine(t, address, payerPair.publicKey, payerPair, data, fstTxHandler);
    await updateTransaction1.assertSuccess(t);
    const updatedCandyMachine = await generated_1.CandyMachine.fromAccountAddress(connection, address);
    (0, spok_1.default)(t, updatedCandyMachine.data, {
        itemsAvailable: (0, utils_1.spokSameBignum)(1000)
    });
    const updatedData = {
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
    const { tx: updateTransaction2 } = await init.updateCandyMachine(t, address, payerPair.publicKey, payerPair, updatedData, fstTxHandler);
    await updateTransaction2.assertError(t);
});
//# sourceMappingURL=03-update.js.map