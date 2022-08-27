import test from 'tape'
import { InitTransactions, killStuckProcess } from './setup/'

const init = new InitTransactions()

killStuckProcess()

test('add_config_lines', async (t) => {
    const { fstTxHandler, payerPair, connection } = await init.payer();
    const items = 100;

    const data = {
        itemsAvailable: items,
        symbol: "CORE",
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
            prefixName: "TEST ",
            nameLength: 10,
            prefixUri: "https://arweave.net/",
            uriLength: 50,
            isSequential: false
        },
        hiddenSettings: null
    };

    const address = await init.createCandyMachine(t, payerPair, data, fstTxHandler, connection);
    const lines = [];

    for (let i = 0; i < items; i++) {
        const line = {
            name: `NFT #${i + 1}`,
            uri: "uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ"
        };

        lines[i] = line;
    }

    await init.addConfigLines(t, address, payerPair, lines, fstTxHandler);
})

test('add_config_lines (hidden settings)', async (t) => {
    const { fstTxHandler, payerPair, connection } = await init.payer();
    const items = 10;

    const data = {
        itemsAvailable: items,
        symbol: "CORE",
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
            name: "Hidden NFT",
            uri: "https://arweave.net/uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ",
            hash: Buffer.from("74bac30d82a0baa41dd2bee4b41bbc36").toJSON().data
        }
    };

    const address = await init.createCandyMachine(t, payerPair, data, fstTxHandler, connection);
    const lines = [];

    for (let i = 0; i < items; i++) {
        const line = {
            name: `NFT #${i + 1}`,
            uri: "uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ"
        };

        lines[i] = line;
    }

    await init.addConfigLines(t, address, payerPair, lines, fstTxHandler, false);
})
