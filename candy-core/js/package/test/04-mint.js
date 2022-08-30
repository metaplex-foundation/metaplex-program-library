"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const tape_1 = __importDefault(require("tape"));
const setup_1 = require("./setup/");
const init = new setup_1.InitTransactions();
(0, setup_1.killStuckProcess)();
(0, tape_1.default)('mint (authority)', async (t) => {
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
    const lines = [];
    for (let i = 0; i < items; i++) {
        const line = {
            name: `NFT #${i + 1}`,
            uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ'
        };
        lines[i] = line;
    }
    const { txs } = await init.addConfigLines(t, address, payerPair, lines, fstTxHandler);
    for (const tx of txs) {
        await fstTxHandler.sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines').assertSuccess(t);
    }
    const { tx: mintTransaction } = await init.mint(t, address, payerPair, fstTxHandler, connection);
    await mintTransaction.assertSuccess(t);
});
(0, tape_1.default)('mint (minter)', async (t) => {
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
    const lines = [];
    for (let i = 0; i < items; i++) {
        const line = {
            name: `NFT #${i + 1}`,
            uri: 'uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ'
        };
        lines[i] = line;
    }
    const { txs } = await init.addConfigLines(t, address, payerPair, lines, fstTxHandler);
    for (const tx of txs) {
        await fstTxHandler.sendAndConfirmTransaction(tx, [payerPair], 'tx: AddConfigLines').assertSuccess(t);
    }
    const { fstTxHandler: minterHandler, minterPair, connection: minterConnection } = await init.minter();
    try {
        const { tx: mintTransaction } = await init.mint(t, address, minterPair, minterHandler, minterConnection);
        await mintTransaction.assertSuccess(t);
        t.fail('only authority is allowed to mint');
    }
    catch (_a) {
        t.pass('minter is not the candy machine authority');
    }
});
//# sourceMappingURL=04-mint.js.map